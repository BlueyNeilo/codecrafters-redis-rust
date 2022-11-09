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
                                        RESPFrame::Bulk(Bytes::from(store_value.as_bytes().to_owned()))
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
                                            Some(value) => RESPFrame::Bulk(Bytes::from(value.as_bytes().to_bytes())),
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
                .parse::<u128>()
                .ok() {
                    match ttl_type.to_ascii_uppercase().as_slice() {
                        b"EX" => set_flags.ttl_flag = Some(SetCommandTTLFlag::EX(ttl as u64)),
                        b"PX" => set_flags.ttl_flag = Some(SetCommandTTLFlag::PX(ttl)),
                        b"EXAT" => set_flags.ttl_flag = Some(SetCommandTTLFlag::EXAT(ttl as u64)),
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
