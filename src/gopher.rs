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

/// Attempts to send a `Request` to a Gopher server and receive its `Response`
/// 
/// # Arguments
/// * `request`: Request to be sent to the server
/// 
/// # Returns
/// A `Response` from the server if sucessfull. Otherwise, returns the appropriate
/// IO error.
pub fn send_and_recv(request: &Request) -> std::io::Result<Response> {
    // Attempts to connect to the Gopher server
    let mut stream = match connect(&request.server_details) {
        Ok(stream) => stream,
        Err(error) => match error.kind() {
            io::ErrorKind::InvalidInput => {
                debug_eprintln!("Malformed server details: {error}");
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
    let selector = format!("{}{}", request.selector, CRLF);
    stream.write_all(selector.as_bytes())?; 

    // Receive the request from the Gopher server
    let response = recv(&stream, &request.item_type)?; 
    return Ok(response)
}

/// Attempts to connect to the provided Gopher server.
/// 
/// # Arguments
/// * `server_details`: hostname:port of the server
/// 
/// # Returns
/// A TCP stream if the connection was sucessfull. Returns an IO error
/// otherwise
pub fn connect(server_details: &str) -> std::io::Result<TcpStream> {
    // Get the current local time
    #[allow(unused_variables)]
    let local_time = Local::now();

    // Print for debugging purposes
    debug_println!("[{:02}h:{:02}m:{:02}s]: CONNECTING TO {}", 
        local_time.time().hour(), local_time.time().minute(), local_time.time().second(),
        server_details
    );

    // Resolves the provided server details and attempts to connect to 
    // any socket address. Will attempt to connect for 5 seconds. 
    for socket_addr in server_details.to_socket_addrs()?.collect::<Vec<_>>() {
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

/// Atempts to receiver a response from a Gopher server. 
/// 
/// # Arguments
/// * `stream`: TCP stream connection to a Gopher server
/// * `item_type`: Item type being requested
/// 
/// # Returns
/// A new `Response` if sucessfull. Otherwise, returns an IO error.
fn recv(mut stream: &TcpStream, item_type: &ItemType) -> std::io::Result<Response> {
    let mut buffer = Vec::new();
    let mut chunk = [0; MAX_CHUNK_SIZE];

    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let start = Instant::now();

    loop {
        match stream.read(&mut chunk) {
            // Entire response has been received
            Ok(0) => break, 
            // Read n bytes
            Ok(n) => {
                buffer.extend_from_slice(&chunk[..n]);
                
                // Overall timeout
                if start.elapsed().as_secs() == 5 {
                    debug_eprintln!("File too long");
                    return Ok(Response::new(buffer, ResponseOutcome::FileTooLong));
                }
            }
            Err(error) => {
                match error.kind() {
                    ErrorKind::Interrupted => continue,
                    // Timeout on a single read
                    ErrorKind::TimedOut | ErrorKind::WouldBlock => {
                        debug_eprintln!("Read timed out");
                        return Ok(Response::new(buffer, ResponseOutcome::Timeout));
                    }, 
                    _ => return Err(error),
                }
            }
        }
    }
    // Removes the end line from text and directory items
    if matches!(*item_type, ItemType::Txt) || matches!(*item_type, ItemType::Dir) {
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