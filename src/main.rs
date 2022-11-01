#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::net::{TcpListener, TcpStream};
use std::io::{Result, Write};
use std::str::from_utf8;

mod resp;
use resp::RESPToken;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    match listener.accept() {
        Ok((socket, addr)) => {
            println!("accepted new client: {:?}", addr);
            handle_connection(socket).unwrap();
        },
        Err(e) => println!("couldn't accept client: {:?}", e),
    }
}

fn handle_connection(stream: TcpStream) -> Result<()> {
    // Receive in RESP, Respond in RESP

    // Hardcode PONG response
    let pong_response: String = RESPToken::SimpleString("PONG").to_string();
    reply(stream, pong_response.as_bytes())?;

    Ok(())
}

fn reply(mut stream: TcpStream, buf: &[u8]) -> Result<()> {
    println!("Sending: {:?}", from_utf8(buf).unwrap());
    stream.write(buf)?;

    Ok(())
}
