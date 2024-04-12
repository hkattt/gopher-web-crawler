use std::io::{Read, Write};
use std::net::TcpStream;
use chrono::{Local, Timelike};

// Open server on Gophie with: comp3310.ddns.net:70
// Local host: 127.0.0.1

// const BUFFER_SIZE: u32  = 4096; // TODO: Is this needed?
const CRLF: &str           = "\r\n";
const TAB:  &str           = "\t";
const COLON: &str          = ":";
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

    fn get_server_details(&self) -> String {
        [self.server_name, COLON, self.server_port].concat()
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
        process_response_line(line)?;
    }
    
    Ok(())
}

fn process_response_line(line: &str) -> std::io::Result<()> {
    if line.starts_with('1') {
        let response_line = ResponseLine::new(line);
        let stream = send_response_line(response_line);
        match stream {
            Ok(stream) => {
                let buffer = recv_response_line(stream)?;
                //println!("DOCUMENT:\n{buffer}");
            }
            Err(e) => println!("Error sending response line: {e}")
        }
    }

    Ok(())
}

fn send_response_line(response_line: ResponseLine) -> std::io::Result<TcpStream> {
    // Get the current local time
    let local_time = Local::now();

    let server_details = response_line.get_server_details();

    let mut stream = TcpStream::connect(server_details)?;

    println!("[{:02}h:{:02}m:{:02}s]: REQUESTING {} FROM {} AT {}", 
        local_time.time().hour(), local_time.time().minute(), local_time.time().second(),
        response_line.selector, response_line.server_name, response_line.server_port
    );

    let selector = [response_line.selector, CRLF].concat();
    stream.write(selector.as_bytes())?;

    Ok(stream)
}

fn recv_response_line(mut stream: TcpStream) -> std::io::Result<String> {
    let mut buffer = String::new();
    stream.read_to_string(&mut buffer)?;
    Ok(buffer)
}