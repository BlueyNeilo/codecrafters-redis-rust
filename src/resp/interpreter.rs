use std::{str::from_utf8, sync::Arc};

use bytes::Bytes;
use tokio::sync::Mutex;

use super::{frame::RESPFrame, command::RedisCommand, super::store::RedisStore};

/**
 * Interprets RESP frames and talks to redis store interface
 */
pub struct RESPInterpreter {
    store: Arc<Mutex<RedisStore>>
}

impl RESPInterpreter {
    pub fn new(store: Arc<Mutex<RedisStore>>) -> Self {
        Self { store: Arc::clone(&store) }
    }
    
    pub async fn interpret(&self, frame: &RESPFrame) -> RESPFrame {
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
                                    let store = self.store.lock().await;
                                    if let Some(store_value) = store.get(bytes_to_string(key)) {
                                        RESPFrame::Bulk(Bytes::from(store_value.as_bytes().to_owned()))
                                    } else {
                                        RESPFrame::Null
                                    }
                                } else {
                                    RESPFrame::Null
                                }
                            },
                            RedisCommand::SET => {
                                if let [RESPFrame::Bulk(key), RESPFrame::Bulk(value)] = args {
                                    let mut store = self.store.lock().await;
                                    store.set(bytes_to_string(key), bytes_to_string(value));

                                    RESPFrame::Simple("OK".to_owned())
                                } else {
                                    pong_response
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
}

fn bytes_to_string(bytes: &Bytes) -> String {
    from_utf8(bytes).unwrap().to_owned()
}
