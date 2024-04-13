mod gopher;

use std::{str, cmp::{max, min}, fs::{self, File}, io::{Error, ErrorKind, Write}, path::Path};

use crate::{CRLF, DOT, MAX_FILENAME_LEN, OUTPUT_FOLDER, SERVER_NAME};
use gopher::{request::Request, response_line::{ItemType, ResponseLine}};

pub struct Crawler {
    ndir: u32,          // The number of directories
    ntxt:  u32,         // The number of simple text files
    // TODO: List of all simple text tiles (full path)
    nbin:  u32,         // The number of binary (i.e. non-text) files
    // TODO: List of all binary files (full path)
    smallest_contents: String,
    largest_txt: u64,   // The size of the largest text file
    smallest_bin: u64,   // The size of the smallest binary file
    largest_bin: u64,    // The size of the largest binary file
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
            smallest_bin: u64::MAX, // TODO: Is there a better way to do this?
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

        match fs::create_dir(Path::new(&OUTPUT_FOLDER)) {
            Ok(_) => (),
            Err(error) => match error.kind() {
                ErrorKind::AlreadyExists => (),
                _ => panic!("Unable to create output folder")
            }
        }
        
        // TODO: Actually handle errors
        let bytes = match gopher::send_and_recv(request){
            Ok(buffer) => buffer,
            Err(error) => {
                match error.kind() {
                    _ => panic!("Problem sending OR receving request")
                }
            }
        };
        // Convert byte stream into a string (i.e. UTF-8 sequence)
        let buffer = match str::from_utf8(&bytes) {
            Ok(buffer) => buffer,
            Err(error) => panic!("Ivalid UTF-8 sequence: {error}"),
        };

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

        // TODO: Delete out

        Ok(())
    }

    fn process_response_line(&mut self, line: &str) -> std::io::Result<()> {
        // TODO: Do this better?
        if line.eq(DOT) {
            return Ok(())
        }
        // TODO: Handle the None case better?
        let response_line = match ResponseLine::new(line) {
            Some(response_line) => {response_line},
            None => return Err(Error::new(ErrorKind::Other, "Malformed response line")),
        };
    
        match response_line.item_type {
            ItemType::TXT => {
                if self.has_crawled(response_line.selector) { return Ok(()) }

                self.used_selectors.push(response_line.selector.to_string());

                let request = Request::new(response_line.selector, 
                        response_line.server_name, 
                        response_line.server_port.parse().unwrap()
                );
                
                // TODO: Actually handle errors?
                // TODO: Deal with big size?
                let buffer = match gopher::send_and_recv(request) {
                    Ok(bytes) => bytes, 
                    Err(error) => {
                        eprintln!("Error sending or receving TXT file: {error}");
                        return Err(error)
                    },
                };

                let f = match Crawler::download_file(response_line.selector, &buffer) {
                    Ok(f) => f, 
                    Err(error) => {
                        eprintln!("Error downloading TXT file: {error}");
                        return Err(error)
                    },
                };
                match f.metadata() {
                    Ok(metadata) => self.largest_txt = max(self.largest_txt, metadata.len()),
                    Err(error) => {
                        eprintln!("Error accessing TXT file metadata: {error}");
                        return Err(error)
                    },
                }
                
                self.ntxt += 1;
            },
            ItemType::DIR => {
                if response_line.server_name != SERVER_NAME {
                    // TODO: Handle external servers
                    // Only need to try connecting
                }

                if self.has_crawled(response_line.selector) { return Ok(()) }
                
                self.used_selectors.push(response_line.selector.to_string());

                self.ndir += 1;

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
                
                let request = Request::new(response_line.selector, 
                    response_line.server_name, 
                    response_line.server_port.parse().unwrap()
                );

                // TODO: Actually handle errors?
                // TODO: Deal with big size?
                let buffer = match gopher::send_and_recv(request) {
                    Ok(buffer) => buffer, 
                    Err(error) => {
                        eprintln!("Error sending or receving BIN file: {error}");
                        return Err(error)
                    },
                };

                let f = match Crawler::download_file(response_line.selector, &buffer) {
                    Ok(f) => f, 
                    Err(error) => {
                        eprintln!("Error downloading BIN file: {error}");
                        return Err(error)
                    },
                };
                match f.metadata() {
                    Ok(metadata) => {
                        self.smallest_bin = min(self.smallest_bin, metadata.len());
                        self.largest_bin = max(self.largest_bin, metadata.len());
                    },
                    Err(error) => {
                        eprintln!("Error accessing BIN file metadata: {error}");
                        return Err(error)
                    },
                }

                self.nbin += 1;
            },
            ItemType::UNKNOWN => {
                // TODO: Should we do stuff with this?
                return Ok(())
            },
        }
        Ok(())
    }

    fn download_file(selector: &str, buffer: &[u8]) -> std::io::Result<File> {
        // Remove the / prefix from the selector. Truncate long selector names
        let file_name = &selector[1..min(selector.len(), MAX_FILENAME_LEN + 1)];
        // Replace forward slashes with dashes to create a valid file name
        let file_name = file_name.replace("/", "-");
        // TODO: Replace the string stuff with global variables?
        let file_path = [OUTPUT_FOLDER, "/", &file_name].concat();
        let mut f = match File::create(file_path) {
            Ok(f) => f, 
            Err(error) => return Err(error),
        };
        f.write_all(buffer)?;
        Ok(f)
    }

    fn has_crawled(&self, selector: &str) -> bool {
        self.used_selectors.iter().any(|used_selector| used_selector == selector)
    }
}