use std::str::from_utf8;

use bytes::{Bytes, Buf};

#[allow(dead_code)]
#[derive(Debug)]
pub enum RESPToken {
    SimpleString(String),       // "+<STRING>\r\n"
    Error(String),              // "-<STRING>\r\n"
    Integer(i64),               // ":<INT>\r\n"
    BulkString(u32, Bytes),     // "$<SIZE>\r\n<STRING>\r\n"
    Null,                       // "$-1\r\n"
    ArraySize(u32)              // "*<SIZE>\r\n"
}

impl RESPToken {
    pub fn to_string(self) -> String {
        match self {
            RESPToken::SimpleString(s) => format!("+{}\r\n", s),
            RESPToken::Error(s) => format!("-{}\r\n", s),
            RESPToken::Integer(n) => format!(":{}\r\n", n),
            RESPToken::BulkString(size, s) => {
                format!("${}\r\n{}\r\n", size, from_utf8(s.bytes()).unwrap())
            },     
            RESPToken::Null => format!("$-1\r\n"),
            RESPToken::ArraySize(size) => format!("*{}\r\n", size),
        }
    }
}
