/**
 * RESP - Redis Serialisation Protocol
 * https://redis.io/docs/reference/protocol-spec/
**/

#[allow(dead_code)]
pub enum RESPToken<'a> {
    SimpleString(&'a str),      // "+<STRING>\r\n"
    Error(&'a str),             // "-<STRING>\r\n"
    Integer(i64),               // ":<INT>\r\n"
    BulkString(u32, &'a str),   // "$<SIZE>\r\n<STRING>\r\n"
    Null,                       // "$-1\r\n"
    ArraySize(u32)              // "*<SIZE>"
}

impl <'a> RESPToken<'a> {
    pub fn to_string(self) -> String {
        match self {
            RESPToken::SimpleString(s) => format!("+{}\r\n", s),
            _ => unimplemented!(),
        }
    }
}
