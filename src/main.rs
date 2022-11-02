#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::io::{Result, Write};
use std::str::from_utf8;

mod resp;
use resp::{token::RESPToken, parser::RESPParser};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    println!("Listening from localhost:6379");

    match listener.accept() {
        Ok((socket, addr)) => {
            println!("accepted new client: {:?}", addr);
            
            handle_connection(socket).unwrap_or_else(|err| {
                println!("Connection closed unexpectedly: '{}'", err)
            });
        },
        Err(e) => println!("couldn't accept client: {:?}", e),
    }
}

fn handle_connection(mut stream: TcpStream) -> Result<()> {
    loop {
        // Receive in RESP, Respond in RESP
        let mut reader = BufReader::new(&stream);
        let request = RESPParser::parse::<&TcpStream>(&mut reader)
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
        reply(&mut stream, pong_response.as_bytes())?;
    }

    Ok(())
}

fn reply(mut stream: &TcpStream, buf: &[u8]) -> Result<()> {
    println!("Sending: {:?}", from_utf8(buf).unwrap());
    stream.write(buf)?;

    Ok(())
}
