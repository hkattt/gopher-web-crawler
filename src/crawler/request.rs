use std::io::Write;
use std::net::TcpStream;
use chrono::{Local, Timelike};

use crate::{CRLF, COLON};

pub struct Request<'a> {
    selector: &'a str, 
    server_details: String,
}

impl<'a> Request<'a> {
    pub fn new(selector: &'a str, server_name: &'a str, server_port: u16) -> Request<'a> {
        let server_details = [server_name, COLON, &server_port.to_string()].concat();
        
        Request {
            selector,
            server_details,
        }
    }

    pub fn send(&self) -> std::io::Result<TcpStream> {
        // Get the current local time
        let local_time = Local::now();
    
        let server_details = &self.server_details;
    
        let mut stream = TcpStream::connect(server_details)?;
    
        println!("[{:02}h:{:02}m:{:02}s]: REQUESTING {} FROM {}", 
            local_time.time().hour(), local_time.time().minute(), local_time.time().second(),
            self.selector, self.server_details
        );
    
        let selector = [self.selector, CRLF].concat();
        stream.write(selector.as_bytes())?;
    
        Ok(stream)
    }
}