#[allow(dead_code)]
#[derive(Debug)]
pub enum RESPToken {
    SimpleString(String),       // "+<STRING>\r\n"
    Error(String),              // "-<STRING>\r\n"
    Integer(i64),               // ":<INT>\r\n"
    BulkString(u32, String),    // "$<SIZE>\r\n<STRING>\r\n"
    Null,                       // "$-1\r\n"
    ArraySize(u32)              // "*<SIZE>"
}

impl RESPToken {
    pub fn to_string(self) -> String {
        match self {
            RESPToken::SimpleString(s) => format!("+{}\r\n", s),
            _ => unimplemented!(),
        }
    }
}
