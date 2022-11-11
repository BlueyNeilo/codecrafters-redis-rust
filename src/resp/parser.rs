use std::{io, num};
use tokio::io::AsyncBufReadExt;
use super::token::RESPToken;
use bytes::Bytes;

pub type RESPMessage = Vec<RESPToken>;

#[derive(Debug)]
pub enum RESPParserError {
    BadIntParse(num::ParseIntError),
    BadRead(io::Error),
    InvalidToken(String),
    InvalidArray(usize)
}

impl From<num::ParseIntError> for RESPParserError {
    fn from(err: num::ParseIntError) -> RESPParserError {
        RESPParserError::BadIntParse(err)
    }
}

impl From<io::Error> for RESPParserError {
    fn from(err: io::Error) -> RESPParserError {
        RESPParserError::BadRead(err)
    }
}

/**
 * Reads tokens from buffer until a RESP frame is read
 */
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
                "-" => parsed_message.push(
                    RESPToken::Error(RESPParser::trim_token(&token_buf))
                ),
                ":" => parsed_message.push(
                    RESPToken::Integer(RESPParser::trim_token(&token_buf).parse::<i64>()?)
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

pub fn to_string(message: RESPMessage) -> String {
    message.into_iter()
        .map(|token| token.to_string())
        .collect::<Vec<String>>()
        .join("")
}


#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use super::*;
    use rstest::rstest;
    use tokio::io::BufReader;

    #[tokio::test]
    async fn should_parse_simple() {
        let mut reader =
            BufReader::new(Cursor::new("+PING\r\n".to_owned()));

        let simple_string_message: RESPMessage =
            RESPParser::parse(&mut reader).await.unwrap();

        assert!(matches!(
            simple_string_message.as_slice(),
            [RESPToken::SimpleString(s)] if s == "PING"
        ));
    }

    #[tokio::test]
    async fn should_parse_all_types() {
        let mut reader = BufReader::new(Cursor::new(
            "*5\r\n+string\r\n-error\r\n:10\r\n$-1\r\n$4\r\nbulk\r\n".to_owned()
        ));

        let array_message: RESPMessage =
            RESPParser::parse(&mut reader).await.unwrap();

        assert!(matches!(
            array_message.as_slice(),
            [
                RESPToken::ArraySize(5),
                RESPToken::SimpleString(simple),
                RESPToken::Error(error),
                RESPToken::Integer(10),
                RESPToken::Null,
                RESPToken::BulkString(4, bulk),
                
            ] if simple == "string" 
                && error == "error" 
                && bulk == "bulk"
        ));
    }

    #[tokio::test]
    async fn should_only_parse_up_to_array_length() {
        let mut reader = BufReader::new(Cursor::new(
            "*2\r\n+many\r\n+things\r\n+to\r\n+say\r\n".to_owned()
        ));

        let array_message: RESPMessage =
            RESPParser::parse(&mut reader).await.unwrap();

        assert_eq!(3, array_message.len());
        assert!(matches!(
            array_message.as_slice(),
            [
                RESPToken::ArraySize(2),
                RESPToken::SimpleString(first),
                RESPToken::SimpleString(second)
                
            ] if first == "many" && second == "things" 
        ));
    }

    #[tokio::test]
    async fn should_not_parse_empty_buffer() {
        let mut reader =
            BufReader::new(Cursor::new("".to_owned()));

        assert!(matches!(
            RESPParser::parse(&mut reader).await.err().unwrap(),
            RESPParserError::InvalidToken(_)
        ));
    }

    #[tokio::test]
    async fn should_not_accept_array_token_after_start() {
        let mut reader =
            BufReader::new(Cursor::new("*2\r\n+hi\r\n*1\r\n+nested\r\n".to_owned()));

        assert!(matches!(
            RESPParser::parse(&mut reader).await.err().unwrap(),
            RESPParserError::InvalidArray(_)
        ));
    }

    #[tokio::test]
    async fn should_not_parse_incomplete_array() {
        let mut reader =
            BufReader::new(Cursor::new("*2\r\n+hi\r\n".to_owned()));

        assert!(matches!(
            RESPParser::parse(&mut reader).await.err().unwrap(),
            RESPParserError::InvalidToken(_)
        ));
    }

    #[tokio::test]
    async fn should_not_parse_bad_integer() {
        let mut reader =
            BufReader::new(Cursor::new("*1\r\n:one\r\n".to_owned()));

        assert!(matches!(
            RESPParser::parse(&mut reader).await.err().unwrap(),
            RESPParserError::BadIntParse(_)
        ));
    }

    #[tokio::test]
    async fn should_not_parse_bad_bulk_size() {
        let mut reader =
            BufReader::new(Cursor::new("*1\r\n$-8\r\nnegative\r\n".to_owned()));

        assert!(matches!(
            RESPParser::parse(&mut reader).await.err().unwrap(),
            RESPParserError::BadIntParse(_)
        ));
    }

    #[rstest]
    #[case("$4\r\n","4")]
    #[case("*1\r\n","1")]
    #[case("+\r\n","")]
    #[case("+PING\r\n","PING")]
    fn correctly_trims_token(#[case] token: String, #[case] trimmed: String) {
        assert_eq!(trimmed, RESPParser::trim_token(&token))
    }

    #[test]
    fn resp_message_to_string() {
        assert_eq!(
            "*2\r\n$7\r\nCOMMAND\r\n$4\r\nDOCS\r\n",
            to_string(vec![
                RESPToken::ArraySize(2),
                RESPToken::BulkString(7, Bytes::from("COMMAND".as_bytes())),
                RESPToken::BulkString(4, Bytes::from("DOCS".as_bytes())),
            ])
        );
    }
}
