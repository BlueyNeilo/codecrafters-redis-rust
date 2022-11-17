use bytes::Bytes;

/**
 * Redis CLI commands
 */
pub enum RedisCommand {
    PING,
    ECHO,
    GET,
    SET,
    UNDEFINED
}

impl From<&Bytes> for RedisCommand {
    fn from(command: &Bytes) -> Self {
        match command.to_ascii_uppercase().as_slice() {
            b"PING" => Self::PING,
            b"ECHO" => Self::ECHO,
            b"GET" => Self::GET,
            b"SET" => Self::SET,
            _ => Self::UNDEFINED
        }
    }
}

pub struct SetCommandFlags {
    pub exist_flag: Option<SetCommandExistFlag>,
    pub get_flag: bool,
    pub ttl_flag: Option<SetCommandTTLFlag>,
}

impl Default for SetCommandFlags {
    fn default() -> Self {
        SetCommandFlags { exist_flag: None, get_flag: false, ttl_flag: None }
    }
}

pub enum SetCommandExistFlag {
    NX, // Only set if it doesn't exist
    XX, // Only set if it exists already
}

pub enum SetCommandTTLFlag {
    EX(u64),    // TTL duration (seconds)
    PX(u64),   // TTL duration (milliseconds)
    EXAT(u64),  // Set expiry at exact unix time (seconds)
    PXAT(u64), // Set expiry at exact unix time (milliseconds)
    KEEPTTL,    // Keep existing TTL when setting value
}
