#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::AsyncWrite;
use std::io::Result;
use std::str::from_utf8;

mod resp;
use resp::{token::RESPToken, parser::RESPParser};

#[tokio::main]
async fn main() {
    let mut listener = TcpListener::bind("127.0.0.1:6379")
        .await.expect("Unable to listen to port");
    println!("Listening from localhost:6379");

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                println!("accepted new client: {:?}", addr);

                tokio::spawn(async move {
                    handle_connection(socket).await.unwrap_or_else(|err| {
                        println!("Connection closed unexpectedly: '{}'", err)
                    });
                });
            },
            Err(e) => println!("couldn't accept client: {:?}", e),
        }
    }
}

async fn handle_connection(mut stream: TcpStream) -> Result<()> {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    loop {
        // Receive in RESP, Respond in RESP
        let request = RESPParser::parse(&mut reader).await
            .unwrap_or_else(|err| {
                // Catch parser error and default to empty message
                println!("Parsing error: {:?}",err);
                vec![]
            });
        
        if request.is_empty() {
            println!("Closing connection, empty request received.");
            break
        }

        println!("Request: {:?}", request);

        // Hardcode PONG response
        let pong_response: String = RESPToken::SimpleString("PONG".to_owned()).to_string();
        let response_buf = pong_response.as_bytes();
        reply(&mut writer, response_buf).await?;
    }

    Ok(())
}

async fn reply<W: AsyncWrite + Unpin>(writer: &mut W, buf: &[u8]) -> Result<()> {
    println!("Sending: {:?}", from_utf8(buf).unwrap());
    writer.write_all(buf).await?;

    Ok(())
}
