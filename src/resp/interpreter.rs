use std::str::from_utf8;

use bytes::{Bytes, Buf};

use super::{
    frame::RESPFrame, 
    command::{RedisCommand, SetCommandFlags, SetCommandExistFlag, SetCommandTTLFlag},
    super::store::RedisStore
};

/**
 * Interprets RESP frames and talks to redis store interface
 */
pub struct RESPInterpreter;

impl RESPInterpreter {
    pub async fn interpret(frame: &RESPFrame) -> RESPFrame {
        // Take PING return PONG (also hardcoded for any unimplemented requests)
        let pong_response = RESPFrame::Simple("PONG".to_owned());
        
        match frame {
            RESPFrame::Array(elements) => {
                match elements.as_slice() {
                    [RESPFrame::Bulk(command), args @ ..] => {
                        match command.into() {
                            RedisCommand::PING => pong_response,
                            RedisCommand::ECHO => {
                                if let [RESPFrame::Bulk(message)] = args {
                                    RESPFrame::Bulk(message.to_owned())
                                } else {
                                    pong_response
                                }
                            },
                            RedisCommand::GET => {
                                if let [RESPFrame::Bulk(key)] = args {
                                    let shared_store = RedisStore::get_shared_store();
                                    let mut store = shared_store.lock().await;

                                    if let Some(store_value) = store.get(&bytes_to_string(key)) {
                                        RESPFrame::Bulk(Bytes::from(store_value))
                                    } else {
                                        RESPFrame::Null
                                    }
                                } else {
                                    RESPFrame::Null
                                }
                            },
                            RedisCommand::SET => {
                                if let [RESPFrame::Bulk(key), RESPFrame::Bulk(value), options @ ..] = args {
                                    let set_flags = RESPInterpreter::calculate_set_flags(options);

                                    let shared_store = RedisStore::get_shared_store();
                                    let mut store = shared_store.lock().await;
                                    
                                    let prev_value = if set_flags.get_flag {
                                        store.get(&bytes_to_string(key))
                                    } else { None };

                                    let update_success = store.set(bytes_to_string(key), bytes_to_string(value), &set_flags);

                                    if set_flags.get_flag {
                                        match prev_value {
                                            Some(value) => RESPFrame::Bulk(Bytes::from(value)),
                                            None => RESPFrame::Null,
                                        }
                                    } else {
                                        if update_success { 
                                            RESPFrame::Simple("OK".to_owned())
                                        } else {
                                            RESPFrame::Null
                                        }
                                    }
                                } else {
                                    RESPFrame::Null
                                }
                            },
                            _ => pong_response
                        }
                    },
                    _ => pong_response
                }
            }
            _ => pong_response
        }
    }

    fn calculate_set_flags(options: &[RESPFrame]) -> SetCommandFlags {
        let mut set_flags = SetCommandFlags::default();
        let mut options_2: &[RESPFrame] = &[];
        let mut options_3: &[RESPFrame] = &[];

        if let [RESPFrame::Bulk(exist_option), other_options @ ..] = options {
            options_2 = other_options;

            match exist_option.to_ascii_uppercase().as_slice() {
                b"NX" => set_flags.exist_flag = Some(SetCommandExistFlag::NX),
                b"XX" => set_flags.exist_flag = Some(SetCommandExistFlag::XX),
                _ => options_2 = options,
            }
        }

        if let [RESPFrame::Bulk(get_option), other_options @ ..] = options_2 {
            options_3 = other_options;

            match get_option.to_ascii_uppercase().as_slice() {
                b"GET" => set_flags.get_flag = true,
                _ => options_3 = options_2,
            }
        }

        if let [RESPFrame::Bulk(ttl_type), RESPFrame::Bulk(ttl_bytes)] = options_3 {
            if let Some(ttl)= from_utf8(ttl_bytes.bytes()).unwrap()
                .to_owned()
                .parse::<u64>()
                .ok() {
                    match ttl_type.to_ascii_uppercase().as_slice() {
                        b"EX" => set_flags.ttl_flag = Some(SetCommandTTLFlag::EX(ttl)),
                        b"PX" => set_flags.ttl_flag = Some(SetCommandTTLFlag::PX(ttl)),
                        b"EXAT" => set_flags.ttl_flag = Some(SetCommandTTLFlag::EXAT(ttl)),
                        b"PXAT" => set_flags.ttl_flag = Some(SetCommandTTLFlag::PXAT(ttl)),
                        _ => {}
                    }
                }
        }

        if let [RESPFrame::Bulk(keepttl_option)] = options_3 {
            if keepttl_option.eq_ignore_ascii_case(b"KEEPTTL") {
                set_flags.ttl_flag = Some(SetCommandTTLFlag::KEEPTTL)
            }
        }

        set_flags
    }
}

fn bytes_to_string(bytes: &Bytes) -> String {
    from_utf8(bytes).unwrap().to_owned()
}


#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::clock::{MockClockSession, Clock};

    use super::*;
    use rstest::rstest;

    #[tokio::test]
    async fn should_interpret_non_array_frames() {
        assert!(matches_pong(RESPInterpreter::interpret(&RESPFrame::Simple("Hi".to_owned())).await));
        assert!(matches_pong(RESPInterpreter::interpret(&RESPFrame::Error("Err".to_owned())).await));
        assert!(matches_pong(RESPInterpreter::interpret(&RESPFrame::Integer(-23)).await));
        assert!(matches_pong(RESPInterpreter::interpret(
            &RESPFrame::Bulk(Bytes::from("Hello world!"))
        ).await));
        assert!(matches_pong(RESPInterpreter::interpret(&RESPFrame::Null).await));
    }

    #[tokio::test]
    async fn should_interpret_empty_array() {
        assert!(matches_pong(RESPInterpreter::interpret(&RESPFrame::Array(vec![])).await));
    }

    #[tokio::test]
    async fn should_interpret_ping_command() {
        assert!(matches_pong(RESPInterpreter::interpret(&RESPFrame::Array(vec![
            RESPFrame::Simple("PING".to_owned())
        ])).await));
    }

    #[rstest]
    #[case("hello")]
    #[case("")]
    #[case("two words")]    
    #[tokio::test]
    async fn should_interpret_echo_command(#[case] message: &str) {
        let response = RESPInterpreter::interpret(&RESPFrame::Array(vec![
            RESPFrame::Bulk(Bytes::from("ECHO")),
            RESPFrame::Bulk(Bytes::from(message.to_owned()))
        ])).await;

        assert!(matches!(response, RESPFrame::Bulk(s) if s == message));

        let lower_case_echo_response = RESPInterpreter::interpret(&RESPFrame::Array(vec![
            RESPFrame::Bulk(Bytes::from("echo")),
            RESPFrame::Bulk(Bytes::from(message.to_owned()))
        ])).await;

        assert!(matches_bulk(lower_case_echo_response, &message));
    }

    #[tokio::test]
    async fn should_interpret_get_missing() {
        let response = interpret_get("test_missing_key").await;
        assert!(matches_null(response));
    }

    #[tokio::test]
    async fn should_interpret_set_get() {
        assert!(matches_null(interpret_get("test_setting_key").await));
        assert!(matches_ok(interpret_set("test_setting_key hi").await));
        assert!(matches_bulk(interpret_get("test_setting_key").await, "hi"));
    }

    // TODO: Simulate clock time instead of sleep
    #[tokio::test]
    async fn should_interpret_set_expiry() {
        MockClockSession::new();
        Clock::mock_freeze();

        assert!(matches_null(interpret_get("test_expiry_key").await));

        // Set and expire after 30ms
        assert!(matches_ok(interpret_set("test_expiry_key existing PX 30").await));
        assert!(matches_bulk(interpret_get("test_expiry_key").await, "existing"));

        // Should still exist just before expiry
        Clock::mock_advance(Duration::from_millis(29));
        assert!(matches_bulk(interpret_get("test_expiry_key").await, "existing"));

        // Should be null after expiry
        Clock::mock_advance(Duration::from_millis(1));
        assert!(matches_null(interpret_get("test_expiry_key").await));        
    }

    fn matches_pong(response: RESPFrame) -> bool {
        matches!(response, RESPFrame::Simple(s) if s == "PONG")
    }

    fn matches_ok(response: RESPFrame) -> bool {
        matches!(response, RESPFrame::Simple(s) if s == "OK")
    }

    fn matches_bulk(response: RESPFrame, message: &str) -> bool {
        matches!(response, RESPFrame::Bulk(s) if s == message)
    }

    fn matches_null(response: RESPFrame) -> bool {
        matches!(response, RESPFrame::Null)
    }

    async fn interpret_get(key: &str) -> RESPFrame {
        RESPInterpreter::interpret(&RESPFrame::Array(vec![
            RESPFrame::Bulk(Bytes::from("GET")),
            RESPFrame::Bulk(Bytes::from(key.to_owned()))
        ])).await
    }

    async fn interpret_set(options: &str) -> RESPFrame {
        //let options = options.to_owned();
        let mut set_array = vec![RESPFrame::Bulk(Bytes::from("SET"))];
        let mut options_array = options.split_whitespace()
            .map(|option| RESPFrame::Bulk(Bytes::from(option.to_owned())))
            .collect::<Vec<RESPFrame>>();
        set_array.append(&mut options_array);

        RESPInterpreter::interpret(&RESPFrame::Array(set_array)).await
    }
}
