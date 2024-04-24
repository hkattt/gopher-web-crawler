pub mod request; 
pub mod response;

use std::{
    io::{
        ErrorKind, 
        Read, 
        Write
    },
    net::TcpStream,
    time::{Duration, Instant}
};

// Chrono imports for data-time functionality
use chrono::{Local, Timelike};

use circular_buffer::CircularBuffer; 

use self::{
    request::Request, 
    response::Response
};

use crate::{CRLF, MAX_CHUNK_SIZE};

pub fn send_and_recv(request: Request) -> std::io::Result<Response> {
    // TODO: Actually handle errors
    let stream = send(request)?;
    let response = recv(stream)?;
    Ok(response)
}

fn send(request: Request) -> std::io::Result<TcpStream> {
    // Get the current local time
    let local_time = Local::now();

    // TODO: Use connect_timeout?
    // TODO: What if connect fails
    let mut stream = TcpStream::connect(&request.server_details)?;

    println!("[{:02}h:{:02}m:{:02}s]: REQUESTING {} FROM {}", 
        local_time.time().hour(), local_time.time().minute(), local_time.time().second(),
        request.selector, &request.server_details
    );

    let selector = [request.selector, CRLF].concat();
    stream.write_all(selector.as_bytes())?;

    Ok(stream)
}

fn recv(mut stream: TcpStream) -> std::io::Result<Response> {
    let mut buffer = Vec::new();
    let mut chunk = [0; MAX_CHUNK_SIZE];
    let mut valid = true;

    let mut circular_buffer = CircularBuffer::<5, u8>::new();
    // TODO: Handle error
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;

    let start = Instant::now();
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break, 
            Ok(n) => {
                let mut end = n;
                for i in (0..n).rev() {
                    circular_buffer.push_back(chunk[i]);
                    // Found \r\n.\r\n
                    if circular_buffer == [b'\n', b'\r', b'.', b'\n', b'\r'] {
                        end = i + 2;
                        break;
                    }
                }
                buffer.extend_from_slice(&chunk[..end]);

                if start.elapsed().as_secs() == 5 {
                    eprintln!("Read timed out");
                    valid = false;
                    break;
                }
            }
            Err(error) => {
                match error.kind() {
                    ErrorKind::Interrupted => continue,
                    ErrorKind::TimedOut | ErrorKind::WouldBlock => {
                        eprintln!("Read timed out");
                        valid = false;
                        break;
                    }, 
                    _ => return Err(error),
                }
            }
        }
    }
    Ok(Response::new(buffer, valid))
}