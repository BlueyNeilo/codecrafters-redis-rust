use bytes::Buf;

use super::frame::RESPFrame;

/**
 * Interprets RESP frames and talks to redis storage interface
 */
pub struct RESPInterpreter;

impl RESPInterpreter {
    pub fn interpret(frame: &RESPFrame) -> RESPFrame {
        // Take PING return PONG (also hardcoded for any unimplemented requests)
        let pong_response = RESPFrame::Simple("PONG".to_owned());
        
        match frame {
            RESPFrame::Array(elements) => {
                match elements.as_slice() {
                    [RESPFrame::Bulk(command), RESPFrame::Bulk(value)] => {
                        match command.bytes() {
                            b"ECHO" => RESPFrame::Bulk(value.to_owned()),
                            _ => pong_response
                        }
                    },
                    [RESPFrame::Bulk(command)] => {
                        match command.bytes() {
                            b"PING" => pong_response,
                            _ => pong_response
                        }
                    }
                    _ => pong_response
                }
            }
            _ => pong_response
        }
    }
}
