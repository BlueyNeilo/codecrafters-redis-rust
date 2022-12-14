use std::str::from_utf8;

use bytes::{Bytes, Buf};

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
                format!("${}\r\n{}\r\n", size, from_utf8(s.chunk()).unwrap())
            },     
            RESPToken::Null => format!("$-1\r\n"),
            RESPToken::ArraySize(size) => format!("*{}\r\n", size),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("PING", "+PING\r\n")]
    #[case("", "+\r\n")]
    #[case(" ", "+ \r\n")]
    #[case("Hello world", "+Hello world\r\n")]
    fn should_serialise_simple_string(#[case] simple: &str, #[case] expected_str: &str) {
        assert_eq!(expected_str, RESPToken::SimpleString(simple.to_owned()).to_string())
    }

    #[rstest]
    #[case("ERR", "-ERR\r\n")]
    #[case("ERR bad message", "-ERR bad message\r\n")]
    #[case("", "-\r\n")]
    fn should_serialise_error(#[case] error: &str, #[case] expected_str: &str) {
        assert_eq!(expected_str, RESPToken::Error(error.to_owned()).to_string())
    }

    #[rstest]
    #[case(0, ":0\r\n")]
    #[case(-10, ":-10\r\n")]
    #[case(23, ":23\r\n")]
    fn should_serialise_int(#[case] int: i64, #[case] expected_str: &str) {
        assert_eq!(expected_str, RESPToken::Integer(int).to_string())
    }

    #[rstest]
    #[case(0, "", "$0\r\n\r\n")]
    #[case(1, "a", "$1\r\na\r\n")]
    #[case(5, "hello", "$5\r\nhello\r\n")]
    #[case(12, "hello world!", "$12\r\nhello world!\r\n")]
    fn should_serialise_bulk_string(
        #[case] size: u32,
        #[case] bulk: &str,
        #[case] expected_str: &str
    ) {
        assert_eq!(
            expected_str,
            RESPToken::BulkString(size, Bytes::from(bulk.to_owned())).to_string()
        )
    }

    #[test]
    fn should_serialise_null() {
        assert_eq!("$-1\r\n", RESPToken::Null.to_string())
    }

    #[rstest]
    #[case(0,"*0\r\n")]
    #[case(1,"*1\r\n")]
    #[case(5,"*5\r\n")]
    fn should_serialise_array(#[case] size: u32, #[case] expected_str: &str) {
        assert_eq!(expected_str, RESPToken::ArraySize(size).to_string())
    }
}
