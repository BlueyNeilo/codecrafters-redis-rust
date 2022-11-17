use std::str::from_utf8;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::AsyncWrite;
use std::io::Result;

use crate::resp::{self, interpreter::RESPInterpreter, parser::{RESPParser, RESPMessage}};
use crate::store::RedisStore;


pub async fn init() {
    tokio::spawn(async {
        RedisStore::init();
        println!("Server initialised")
    });
}

pub async fn listen() {
    let listener = TcpListener::bind("127.0.0.1:6379")
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

        let response_message: RESPMessage = RESPInterpreter::interpret(&(request.into())).await.into();
        println!("Response: {:?}", response_message);

        let response_string: String = resp::parser::to_string(response_message);
        reply(&mut writer, response_string.as_bytes()).await?;
    }

    Ok(())
}

async fn reply<W: AsyncWrite + Unpin>(writer: &mut W, buf: &[u8]) -> Result<()> {
    println!("Sending: {:?}", from_utf8(buf).unwrap());
    writer.write_all(buf).await?;

    Ok(())
}
