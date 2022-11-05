use bytes::Bytes;

/**
 * Redis CLI commands
 */
pub enum RedisCommand {
    PING,
    ECHO,
    UNDEFINED
}

impl From<&Bytes> for RedisCommand {
    fn from(command: &Bytes) -> Self {
        match command.to_ascii_uppercase().as_slice() {
            b"PING" => Self::PING,
            b"ECHO" => Self::ECHO,
            _ => Self::UNDEFINED
        }
    }
}
