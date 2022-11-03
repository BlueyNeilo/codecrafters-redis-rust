use std::{io, num::ParseIntError};
use tokio::io::AsyncBufReadExt;
use super::token::RESPToken;
use bytes::Bytes;

pub type RESPMessage = Vec<RESPToken>;

#[derive(Debug)]
pub enum RESPParserError {
    BadIntParse(ParseIntError),
    BadRead(io::Error),
    InvalidToken(String),
    InvalidArray(usize)
}

impl From<ParseIntError> for RESPParserError {
    fn from(err: ParseIntError) -> RESPParserError {
        RESPParserError::BadIntParse(err)
    }
}

impl From<io::Error> for RESPParserError {
    fn from(err: io::Error) -> RESPParserError {
        RESPParserError::BadRead(err)
    }
}

pub struct RESPParser;

impl RESPParser {
    // Parses one token (or array of tokens) at time
    pub async fn parse<R: AsyncBufReadExt + Unpin>(
        reader: &mut R
    ) -> Result<RESPMessage, RESPParserError> {
        let mut parsed_message = vec![];
        let mut remaining_tokens = 1;
        let mut token_buf: String = String::new();
    
        while remaining_tokens > 0 {
            RESPParser::read_token(reader, &mut token_buf).await?;
            let prefix = token_buf.get(..1)
                .ok_or(RESPParserError::InvalidToken(token_buf.to_owned()))?;

            match prefix {
                "+" => parsed_message.push(
                    RESPToken::SimpleString(RESPParser::trim_token(&token_buf))
                ),
                "$" => {
                    let string_size = RESPParser::trim_token(&token_buf);
                    if string_size == "-1" {
                        parsed_message.push(RESPToken::Null)
                    } else {
                        let string_size = string_size.parse::<u32>()?;

                        RESPParser::read_token(reader, &mut token_buf).await?;
                        let bulk_string = Bytes::from(token_buf.trim().to_owned());
                        
                        parsed_message.push(
                            RESPToken::BulkString(string_size, bulk_string)
                        )
                    }
                },
                "*" => {
                    // Only accept array at start of message
                    if !parsed_message.is_empty() {
                        return Err(RESPParserError::InvalidArray(parsed_message.len()))
                    }

                    let array_size = RESPParser::trim_token(&token_buf)
                        .parse::<u32>()?;
                    parsed_message.push(RESPToken::ArraySize(array_size));
                    remaining_tokens += array_size;
                },
                _ => {
                    return Err(RESPParserError::InvalidToken(token_buf));
                }
            }
            remaining_tokens -= 1
        }
    
        Ok(parsed_message)
    }
    
    async fn read_token<R: AsyncBufReadExt + Unpin>(
        reader: &mut R,
        token_buf: &mut String
    ) -> io::Result<()> {
        token_buf.clear();
        reader.read_line(token_buf).await?;

        println!("Read token: {:?}", token_buf);
        Ok(())
    }

    fn trim_token(token: &String) -> String {
        token.get(1..).unwrap().trim().to_owned()
    }
}
