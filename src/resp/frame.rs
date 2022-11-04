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
            RESPToken::Integer(i) => RESPFrame::Integer(i),
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
            RESPFrame::Integer(i) => RESPToken::Integer(i),
            RESPFrame::Bulk(s) => RESPToken::BulkString(s.len() as u32, s),
            RESPFrame::Null => RESPToken::Null,
            RESPFrame::Array(_) => unimplemented!("RESP Frame array doesn't map to a token"),
        }
    }
}
