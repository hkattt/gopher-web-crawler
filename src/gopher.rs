pub mod request; 
pub mod response;

use::debug_print::{debug_println, debug_eprintln};

use std::{
    io::{
        self, 
        ErrorKind, 
        Read, 
        Write
    }, 
    net::{TcpStream, ToSocketAddrs}, 
    time::{Duration, Instant}
};

// Chrono imports for data-time functionality
use chrono::Local;
#[allow(unused_imports)]
use chrono::Timelike;

use self::{
    request::Request, 
    response::{ItemType, Response, ResponseOutcome}
};

use crate::{CRLF, MAX_CHUNK_SIZE};

pub fn send_and_recv(request: &Request) -> std::io::Result<Response> {
    let mut stream = match connect(&request.server_details) {
        Ok(stream) => stream,
        Err(error) => match error.kind() {
            io::ErrorKind::InvalidInput => {
                debug_eprintln!("Malformed server details: {error}");
                // TODO: Different outcome for this?
                return Ok(Response {buffer: Vec::new(), response_outcome: ResponseOutcome::ConnectionFailed})
            },
            io::ErrorKind::AddrNotAvailable => {
                debug_eprintln!("Provided host or port is not available: {error}");
                return Ok(Response {buffer: Vec::new(), response_outcome: ResponseOutcome::ConnectionFailed})
            },
            _ => return Err(error),
        }
    };

    // Get the current local time
    let local_time = Local::now();
    
    println!("[{:02}h:{:02}m:{:02}s]: REQUESTING {} FROM {}", 
        local_time.time().hour(), local_time.time().minute(), local_time.time().second(),
        request.selector, &request.server_details
    );

    // Send the request to the Gopher server
    let selector = [request.selector, CRLF].concat(); // TODO: Use format
    stream.write_all(selector.as_bytes())?; // TODO: Handle error

    // Receive the request from the Gopher server
    let response = recv(&stream, &request.item_type)?; // TODO: Handle error
    return Ok(response)
}

pub fn connect(server_details: &str) -> std::io::Result<TcpStream> {
    // Get the current local time
    #[allow(unused_variables)]
    let local_time = Local::now();

    debug_println!("[{:02}h:{:02}m:{:02}s]: CONNECTING TO {}", 
        local_time.time().hour(), local_time.time().minute(), local_time.time().second(),
        server_details
    );

    // TODO: Handle this error?
    let socket_addrs: Vec<_> = server_details.to_socket_addrs()?.collect();

    for socket_addr in socket_addrs {
        // TODO: Set global timeout variable
        match TcpStream::connect_timeout(&socket_addr, Duration::from_secs(5)) {
            Ok(stream) => return Ok(stream),
            Err(_) => continue
        };
    }
    Err(io::Error::new(
        io::ErrorKind::AddrNotAvailable,
        "Unable to connect to provided hostname and port"
    ))
}

fn recv(mut stream: &TcpStream, item_type: &ItemType) -> std::io::Result<Response> {
    let mut buffer = Vec::new();
    let mut chunk = [0; MAX_CHUNK_SIZE];

    // TODO: Handle error
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;

    let start = Instant::now();
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break, 
            Ok(n) => {
                buffer.extend_from_slice(&chunk[..n]);

                if start.elapsed().as_secs() == 5 {
                    debug_eprintln!("File too long");
                    return Ok(Response::new(buffer, ResponseOutcome::FileTooLong));
                }
            }
            Err(error) => {
                match error.kind() {
                    ErrorKind::Interrupted => continue,
                    ErrorKind::TimedOut | ErrorKind::WouldBlock => {
                        debug_eprintln!("Read timed out");
                        return Ok(Response::new(buffer, ResponseOutcome::Timeout));
                    }, 
                    _ => return Err(error),
                }
            }
        }
    }

    if matches!(*item_type, ItemType::TXT) || matches!(*item_type, ItemType::DIR){
        if buffer.len() < 3 {
            Ok(Response::new(buffer, ResponseOutcome::MissingEndLine))
        } else if buffer.iter().rev().take(3).eq(&[b'\n', b'\r', b'.']) {
            buffer.truncate(buffer.len() - 3);
            Ok(Response::new(buffer, ResponseOutcome::Complete))
        } else {
            Ok(Response::new(buffer, ResponseOutcome::MissingEndLine))
        }
    } else {
        Ok(Response::new(buffer, ResponseOutcome::Complete))
    }
}