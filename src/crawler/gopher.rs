use std::io::{Read, Write};
use std::net::TcpStream;
use chrono::{Local, Timelike};

use request::Request;
use crate::CRLF;

pub mod request;
pub mod response_line;

pub fn send_and_recv(request: Request) -> std::io::Result<String> {
    // TODO: Actually handle errors
    let stream = send(request)?;
    let response = recv(stream)?;
    Ok(response)
}

fn send(request: Request) -> std::io::Result<TcpStream> {
    // Get the current local time
    let local_time = Local::now();

    let mut stream = TcpStream::connect(&request.server_details)?;

    println!("[{:02}h:{:02}m:{:02}s]: REQUESTING {} FROM {}", 
        local_time.time().hour(), local_time.time().minute(), local_time.time().second(),
        request.selector, &request.server_details
    );

    let selector = [request.selector, CRLF].concat();
    stream.write(selector.as_bytes())?;

    Ok(stream)
}

fn recv(mut stream: TcpStream) -> std::io::Result<String> {
    let mut buffer = String::new();
    stream.read_to_string(&mut buffer)?;
    Ok(buffer)
}