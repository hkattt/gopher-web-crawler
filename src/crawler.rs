use std::io::{Read, Error, ErrorKind};

use request::Request;
use response_line::{ItemType, ResponseLine};
use crate::{CRLF, DOT, SERVER_NAME};

mod request;
mod response_line;

pub struct Crawler {
    ndir: u32,          // The number of directories
    ntxt:  u32,         // The number of simple text files
    // TODO: List of all simple text tiles (full path)
    nbin:  u32,         // The number of binary (i.e. non-text) files
    // TODO: List of all binary files (full path)
    smallest_contents: String,
    largest_txt: u32,   // The size of the largest text file
    smallest_bin: u32,   // The size of the smallest binary file
    largest_bin: u32,    // The size of the largest binary file
    nerr: u32,           // The number of unique invalid references (error types)
    // TODO: A list of external servers (see spec)
    // TODO: Any references that have "issues/errors" that your code needs to explicity deal with
    used_selectors: Vec<String>,
}

impl Default for Crawler {
    fn default() -> Crawler {
        Crawler {
            ndir: 1,   // Count the root directory
            ntxt:  0,
            // TODO: List of all simple text tiles (full path)
            nbin:  0,
            // TODO: List of all binary files (full path)
            smallest_contents: String::new(),
            largest_txt: 0, 
            smallest_bin: u32::MAX, // TODO: Is there a better way to do this?
            largest_bin: 0,
            nerr: 0,
            // TODO: Add to this
            used_selectors: Vec::new(),
        }
    }
}

impl Crawler {
    pub fn new() -> Crawler {
        Crawler { ..Default::default() }
    }

    pub fn crawl(&mut self, selector: &str, server_name: &str, server_port: u16) -> std::io::Result<()> {
        let request = Request::new(selector, server_name, server_port);
        self.used_selectors.push(selector.to_string());
        
        let stream = request.send();
        let mut buffer = String::new();

        let mut stream = match stream {
            Ok(stream) => stream,
            Err(error) => {
                match error.kind() {
                    _ => panic!("Problem sending request")
                }
            }
        };
        stream.read_to_string(&mut buffer)?;

        let lines = buffer.split(CRLF);

        for line in lines {
            match self.process_response_line(line) {
                Ok(_) => {},
                Err(error) => {
                    match error.kind() {
                        ErrorKind::Other => {},
                        _ => panic!("Problem processing response line")
                    }
                }
            }
        }
        Ok(())
    }

    fn process_response_line(&mut self, line: &str) -> std::io::Result<()> {
        // TODO: Do this better?
        if line.eq(DOT) {
            return Ok(())
        }
        // TODO: Handle the None case better?
        let response_line = ResponseLine::new(line);
        let response_line = match response_line {
            Some(response_line) => {response_line},
            None => return Err(Error::new(ErrorKind::Other, "Malformed response line")),
        };
    
        match response_line.item_type {
            ItemType::TXT => {
                if self.has_crawled(response_line.selector) { return Ok(()) }

                self.used_selectors.push(response_line.selector.to_string());

                self.ntxt += 1;
                // let stream = send_response_line(response_line);
                // match stream {
                //     Ok(stream) => {
                //         let buffer = recv_response_line(stream)?;
                //         //println!("DOCUMENT:\n{buffer}");
                //     }
                //     Err(e) => eprintln!("Error sending response line: {e}")
                // }
            },
            ItemType::DIR => {
                self.ndir += 1;

                if response_line.get_server_details() != SERVER_NAME {
                    // TODO: Handle external servers
                    // Only need to try connecting
                }

                if self.has_crawled(response_line.selector) { return Ok(()) }
                
                self.used_selectors.push(response_line.selector.to_string());

                self.crawl(response_line.selector, 
                    response_line.server_name, 
                    response_line.server_port.parse().unwrap()
                )?;
            },
            ItemType::ERR => {
                self.nerr += 1;
                self.ndir -= 1; // Ignore parent directory that led to the error
            },
            ItemType::BIN => {
                if self.has_crawled(response_line.selector) { return Ok(()) }

                self.used_selectors.push(response_line.selector.to_string());

                self.nbin += 1;
            },
            ItemType::UNKNOWN => {
                // TODO: Should we do stuff with this?
                return Ok(())
            },
        }
        Ok(())
    }

    fn has_crawled(&self, selector: &str) -> bool {
        self.used_selectors.iter().any(|used_selector| used_selector == selector)
    }
}