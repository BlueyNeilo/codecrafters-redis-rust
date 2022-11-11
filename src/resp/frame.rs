use bytes::Bytes;

use super::{parser::RESPMessage, token::RESPToken};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum RESPFrame {
    Simple(String),
    Error(String),
    Integer(i64),
    Bulk(Bytes),
    Null,
    Array(Vec<RESPFrame>)
}

impl From<RESPMessage> for RESPFrame {
    fn from(message: RESPMessage) -> RESPFrame {
        assert!(!message.is_empty(), "Empty RESP message found");

        let mut message_iter = message.into_iter();
        let token = message_iter.next().unwrap();
        let rest = message_iter;

        match token {
            // Nested arrays not supported
            RESPToken::ArraySize(_) => RESPFrame::Array(
                rest.map(|child| child.into()).collect()
            ),
            _ => token.into()
        }
    }
}

impl From<RESPToken> for RESPFrame {
    fn from(token: RESPToken) -> RESPFrame {
        match token {
            RESPToken::SimpleString(s) => RESPFrame::Simple(s),
            RESPToken::Error(s) => RESPFrame::Error(s),
            RESPToken::Integer(n) => RESPFrame::Integer(n),
            RESPToken::BulkString(_, s) => RESPFrame::Bulk(s),
            RESPToken::Null => RESPFrame::Null,
            RESPToken::ArraySize(_) => unreachable!(),
        }
    }
}

impl From<RESPFrame> for RESPMessage {
    fn from(frame: RESPFrame) -> RESPMessage {
        match frame {
            RESPFrame::Array(data) => {
                let mut message = vec![RESPToken::ArraySize(data.len() as u32)];

                for child in data {
                    message.push(child.into())
                }
                message
            }
            _ => vec![frame.into()],
        }
    }
}

impl From<RESPFrame> for RESPToken {
    fn from(frame: RESPFrame) -> RESPToken {
        match frame {
            RESPFrame::Simple(s) => RESPToken::SimpleString(s),
            RESPFrame::Error(s) => RESPToken::Error(s),
            RESPFrame::Integer(n) => RESPToken::Integer(n),
            RESPFrame::Bulk(s) => RESPToken::BulkString(s.len() as u32, s),
            RESPFrame::Null => RESPToken::Null,
            RESPFrame::Array(_) => unimplemented!("RESP Frame array doesn't map to a token"),
        }
    }
}


#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::resp::{token::RESPToken, parser::RESPMessage};

    use super::RESPFrame;
    use rstest::rstest;

    #[test]
    fn should_convert_frame_to_and_from_simple_string(){
        let message = vec![RESPToken::SimpleString("PING".to_owned())];

        let simple_frame = RESPFrame::from(message);
        assert!(matches!(
            &simple_frame,
            RESPFrame::Simple(s) if s == "PING"
        ));

        let back_to_message = RESPMessage::from(simple_frame);
        assert!(matches!(
            back_to_message.as_slice(),
            [RESPToken::SimpleString(s)] if s == "PING"
        ));
    }

    #[test]
    fn should_convert_frame_to_and_from_error_string(){
        let message = vec![RESPToken::Error("ERR".to_owned())];
        
        let error_frame = RESPFrame::from(message);
        assert!(matches!(
            &error_frame,
            RESPFrame::Error(s) if s == "ERR"
        ));

        let back_to_message = RESPMessage::from(error_frame);
        assert!(matches!(
            back_to_message.as_slice(),
            [RESPToken::Error(s)] if s == "ERR"
        ));
    }

    #[rstest]
    #[case(-1)]
    #[case(0)]
    #[case(1)]
    #[case(4000000)]
    fn should_convert_frame_to_and_from_integer(#[case] int: i64){
        let message = vec![RESPToken::Integer(int)];
        
        let int_frame = RESPFrame::from(message);
        assert!(matches!(
            &int_frame,
            RESPFrame::Integer(int_match) if *int_match == int
        ));

        let back_to_message = RESPMessage::from(int_frame);
        assert!(matches!(
            back_to_message.as_slice(),
            [RESPToken::Integer(int_match)] if *int_match == int
        ));
    }

    #[test]
    fn should_convert_frame_to_and_from_null(){
        let message = vec![RESPToken::Null];
        
        let null_frame = RESPFrame::from(message);
        assert!(matches!(
            null_frame,
            RESPFrame::Null
        ));

        let back_to_message = RESPMessage::from(null_frame);
        assert!(matches!(
            back_to_message.as_slice(),
            [RESPToken::Null]
        ));
    }

    #[test]
    fn should_convert_frame_to_and_from_bulk_string(){
        let message = vec![RESPToken::BulkString(4, Bytes::from("BULK"))];
        
        let bulk_frame = RESPFrame::from(message);
        assert!(matches!(
            &bulk_frame,
            RESPFrame::Bulk(s) if s == "BULK"
        ));

        let back_to_message = RESPMessage::from(bulk_frame);
        assert!(matches!(
            back_to_message.as_slice(),
            [RESPToken::BulkString(4, s)] if s == "BULK"
        ));
    }

    #[test]
    fn should_convert_frame_to_and_from_array(){
        let message = vec![
            RESPToken::ArraySize(5),
            RESPToken::SimpleString("PING".to_owned()),
            RESPToken::Error("ERR bad".to_owned()),
            RESPToken::Integer(42),
            RESPToken::BulkString(4, Bytes::from("BULK")),
            RESPToken::Null
        ];
        
        let bulk_frame = RESPFrame::from(message);
        assert!(matches!(
            &bulk_frame,
            RESPFrame::Array(array) if matches!(
                array.as_slice(),
                [
                    RESPFrame::Simple(_),
                    RESPFrame::Error(_),
                    RESPFrame::Integer(_),
                    RESPFrame::Bulk(_),
                    RESPFrame::Null,
                ]
            )
        ));

        let back_to_message = RESPMessage::from(bulk_frame);
        assert!(matches!(
            back_to_message.as_slice(),
            [
                RESPToken::ArraySize(5),
                RESPToken::SimpleString(simple),
                RESPToken::Error(error),
                RESPToken::Integer(42),
                RESPToken::BulkString(4, bulk),
                RESPToken::Null
            ] if simple == "PING"
                && error == "ERR bad"
                && bulk == "BULK"
        ));
    }

    #[test]
    #[should_panic]
    fn bad_empty_message() {
        let _invalid_frame = RESPFrame::from(Vec::<RESPToken>::from([]));
    }

}
