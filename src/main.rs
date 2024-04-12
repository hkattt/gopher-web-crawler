use std::io::{Read, Write};
use std::net::TcpStream;
use chrono::format::Item;
use chrono::{Local, Timelike};

// Open server on Gophie with: comp3310.ddns.net:70
// Local host: 127.0.0.1

// const BUFFER_SIZE: u32  = 4096; // TODO: Is this needed?
const CRLF: &str           = "\r\n";
const TAB: &str            = "\t";
const COLON: &str          = ":";
const DOT: &str            = ".";
const SERVER_DETAILS: &str = "comp3310.ddns.net:70";

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
        println!("Response line: {}", line);
        // TODO: What if there are not 4 tabs? Result NOne
        let mut parts = line.splitn(4, TAB);

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
        }
    }
}

impl Crawler {
    fn crawl(&mut self, server_details: &str) -> std::io::Result<()> {
        let mut stream = TcpStream::connect(server_details)?;

        let selector = String::from(CRLF);
        let mut buffer = String::new();

        stream.write(selector.as_bytes())?;
        stream.read_to_string(&mut buffer)?;

        let lines = buffer.split(CRLF);

        for line in lines {
            self.process_response_line(line)?;
        }
        
        Ok(())
    }

    fn process_response_line(&mut self, line: &str) -> std::io::Result<()> {
        // TODO: Do this better?
        if line.eq(DOT) {
            println!("Encountered dot");
            return Ok(())
        }
        // TODO: Handle the None case better?
        let response_line = ResponseLine::new(line).unwrap();
    
        match response_line.item_type {
            ItemType::TXT => {
                self.ntxt += 1;
                let stream = send_response_line(response_line);
                match stream {
                    Ok(stream) => {
                        //let buffer = recv_response_line(stream)?;
                        //println!("DOCUMENT:\n{buffer}");
                    }
                    Err(e) => eprintln!("Error sending response line: {e}")
                }
            },
            ItemType::DIR => {
                self.ndir += 1;
            },
            ItemType::ERR => {
                self.nerr += 1;
            },
            ItemType::BIN => {
                self.nbin += 1;
            },
            ItemType::UNKNOWN => {
                // TODO: Should we do stuff with this?
            },
        }
        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    let mut crawler = Crawler { ..Default::default() };
    crawler.crawl(SERVER_DETAILS)?;
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