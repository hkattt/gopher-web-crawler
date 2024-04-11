use std::io::{Read, Write};
use std::net::TcpStream;

// Open server on Gophie with: comp3310.ddns.net:70
// Local host: 127.0.0.1

// const BUFFER_SIZE: u32  = 4096; // TODO: Is this needed?
const CRLF: &str           = "\r\n";
const TAB:  &str           = "\t";
const SERVER_DETAILS: &str = "comp3310.ddns.net:70";

struct ResponseLine<'a> {
    selector: &'a str, 
    server_name: &'a str,
    server_port: &'a str,
}

impl<'a> ResponseLine<'a> {
    fn new(line: &'a str) -> ResponseLine<'a> {
        let mut parts = line.splitn(4, TAB);

        let _ = parts.next().unwrap();

        ResponseLine {
            selector: parts.next().unwrap(),
            server_name: parts.next().unwrap(),
            server_port: parts.next().unwrap(),
        }
    }
}

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect(SERVER_DETAILS)?;

    let selector = String::from(CRLF);
    let mut buffer = String::new();

    stream.write(selector.as_bytes())?;
    stream.read_to_string(&mut buffer)?;

    let lines = buffer.split(CRLF);

    for line in lines {
        process_response_line(line)
    }
    
    Ok(())
}

fn process_response_line(line: &str) {
    if line.starts_with('0') {
        let response_line = ResponseLine::new(line);

        println!("--- NEW LINE ---\n \
            selector: {}\n \
            server name: {}\n \
            server_port: {}",
            response_line.selector, 
            response_line.server_name,
            response_line.server_port
        );
    }

    if line.starts_with('1') {
        let response_line = ResponseLine::new(line);

        println!("--- NEW LINE ---\n \
            selector: {}\n \
            server name: {}\n \
            server_port: {}",
            response_line.selector, 
            response_line.server_name,
            response_line.server_port
        );
    }
}
