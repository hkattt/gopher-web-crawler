use std::io::{Read, Write, Error, ErrorKind};
use std::net::TcpStream;
use chrono::{Local, Timelike};

// Open server on Gophie with: comp3310.ddns.net:70
// Local host: 127.0.0.1

// const BUFFER_SIZE: u32  = 4096; // TODO: Is this needed?
const CRLF: &str              = "\r\n";
const TAB: &str               = "\t";
const COLON: &str             = ":";
const DOT: &str               = ".";
const STARTING_SELECTOR: &str = "";
const SERVER_NAME: &str       = "comp3310.ddns.net";
const SERVER_PORT: u16        = 70;

struct Request<'a> {
    selector: &'a str, 
    server_details: String,
}

impl<'a> Request<'a> {
    fn new(selector: &'a str, server_name: &'a str, server_port: u16) -> Request<'a> {
        let server_details = [server_name, COLON, &server_port.to_string()].concat();
        
        Request {
            selector,
            server_details,
        }
    }

    fn send(&self) -> std::io::Result<TcpStream> {
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

enum ItemType {
    TXT,     // 0   Item is a text file
    DIR,     // 1   Item is a directory 
    ERR,     // 3   Item is a error
    BIN,     // 9   Item is a binary file
    UNKNOWN, // _    Item is unknown
}

struct ResponseLine<'a> {
    item_type:   ItemType,
    selector:    &'a str, 
    server_name: &'a str,
    server_port: &'a str,
}

impl<'a> ResponseLine<'a> {
    fn new(line: &'a str) -> Option<ResponseLine<'a>> {
        // TODO: What if there are not 4 tabs? Result NOne
        let mut parts = line.splitn(4, TAB);

        // TODO: Can we do this without cloning?
        if parts.clone().count() != 4 {return None;}

        let user_display_string = parts.next().unwrap();
        
        let mut item_type = ItemType::UNKNOWN;
        match user_display_string.chars().next() {
            Some(i) => match i {
                '0' => item_type = ItemType::TXT,
                '1' => item_type = ItemType::DIR,
                '3' => item_type = ItemType::ERR,
                '9' => item_type = ItemType::BIN,
                _   => ()
            },
            None => return None
        };

        Some(
            ResponseLine {
                item_type,
                selector:    parts.next().unwrap(),
                server_name: parts.next().unwrap(),
                server_port: parts.next().unwrap(),
            }
        )
    }

    fn get_server_details(&self) -> String {
        [self.server_name, COLON, self.server_port].concat()
    }
}

struct Crawler {
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
    fn crawl(&mut self, selector: &str, server_name: &str, server_port: u16) -> std::io::Result<()> {
        let request = Request::new(selector, server_name, server_port);
        self.used_selectors.push(STARTING_SELECTOR.to_string());
        
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

fn main() -> std::io::Result<()> {
    // TODO: Should we make SERVER_PORT a &str?
    let mut crawler = Crawler { ..Default::default() };
    crawler.crawl(STARTING_SELECTOR, SERVER_NAME, SERVER_PORT)?;
    Ok(())
}

fn recv_response_line(mut stream: TcpStream) -> std::io::Result<String> {
    let mut buffer = String::new();
    stream.read_to_string(&mut buffer)?;
    Ok(buffer)
}