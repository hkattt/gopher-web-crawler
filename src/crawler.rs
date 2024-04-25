use std::{
    cmp::min, 
    fs::File, 
    io::Write, 
    str 
};

// Chrono imports for data-time functionality
use chrono::Local;
#[allow(unused_imports)]
use chrono::Timelike;

use::debug_print::{debug_println, debug_eprintln};

use crate::gopher::{
    self, 
    request::Request, 
    response::{ItemType, ResponseLine, ResponseOutcome}
};

use crate::{MAX_FILENAME_LEN, OUTPUT_FOLDER, SERVER_NAME};

pub struct Crawler {
    ndir: u32,                                           // The number of directories
    dirs: Vec<(String, String)>,                         // List of all directories (server details, directory)

    ntxt:  u32,                                          // The number of simple text files
    txt_files: Vec<(String, String)>,                    // List of all simple text tiles (full path) (server details, text file)
    
    nbin:  u32,                                          // The number of binary (i.e. non-text) files
    bin_files: Vec<(String, String)>,                    // List of all binary files (full path) (server details, binary file)
    
    smallest_contents: String,                           // Contents of the smallest text file
    smallest_txt: u64,                                   // The size of the smallest text file
    largest_txt: u64,                                    // The size of the largest text file
    
    smallest_bin: u64,                                   // The size of the smallest binary file
    largest_bin: u64,                                    // The size of the largest binary file
    
    smallest_txt_selector: (String, String),             // The selector of the smallest text file
    largest_txt_selector: (String, String),              // The selector of the largest text file
    smallest_bin_selector: (String, String),             // The selector of the smallest binary file
    largest_bin_selector: (String, String),              // The selector of the largest binary file
    
    nerr: u32,                                           // The number of unique invalid references (error types)
    external_servers: Vec<(String, bool)>,               // List of external servers and if they accepted a connection
    invalid_references: Vec<(String, String, ResponseOutcome)>,  // List of references that have "issues/errors" that had be explicitly dealt with
    used: Vec<(String, u16, String)>,                         // Used (server name, server port, selector)
}

impl Default for Crawler {
    fn default() -> Crawler {
        Crawler {
            ndir: 0,   
            dirs: Vec::new(),
            
            ntxt:  0,
            txt_files: Vec::new(), 
            
            nbin:  0,
            bin_files: Vec::new(),
            
            smallest_contents: String::new(),
            smallest_txt: u64::MAX,
            largest_txt: 0, 
            
            smallest_bin: u64::MAX, 
            largest_bin: 0,
            
            smallest_txt_selector: (String::new(), String::new()),
            largest_txt_selector: (String::new(), String::new()),
            smallest_bin_selector: (String::new(), String::new()),
            largest_bin_selector: (String::new(), String::new()),
            
            nerr: 0,
            external_servers: Vec::new(),
            invalid_references: Vec::new(),
            used: Vec::new(),
        }
    }
}

impl Crawler {
    pub fn new() -> Crawler {
        Crawler { ..Default::default() }
    }

    pub fn report(&self) {
        let server_selector_display = |(server_details, selector): &(String, String)| [server_details, ": ", selector].concat();
        println!(
"\nSTART CRAWLER REPORT\n
\tNumber of Gopher directories: {}
\t\t{}\n
\tNumber of simple text files: {}
\t\t{}\n
\tNumber of binary files: {}
\t\t{}\n
\tSmallest text file: {}
\t\tSize: {} bytes
\t\tContents: {}\n
\tSize of the largest text file: {} bytes
\t\t{}\n
\tSize of the smallest binary file: {} bytes
\t\t{}\n
\tSize of the largest binary file: {} bytes
\t\t{}\n
\tThe number of unique invalid references (error types): {}\n
\tList of external servers: 
\t\t{}\n
\tReferences that have issues/errors: 
\t\t{}\n
END CRAWLER REPORT",
            self.ndir,
            self.dirs.iter()
                .map(server_selector_display)
                .collect::<Vec<String>>()
                .join("\n\t\t"),
            self.ntxt,
            self.txt_files.iter()
                .map(server_selector_display)
                .collect::<Vec<String>>()
                .join("\n\t\t"),
            self.nbin,
            self.bin_files.iter()
                .map(server_selector_display)
                .collect::<Vec<String>>()
                .join("\n\t\t"),
            server_selector_display(&self.smallest_txt_selector),
            self.smallest_txt,
            self.smallest_contents,
            self.largest_txt,
            server_selector_display(&self.largest_txt_selector),
            self.smallest_bin,
            server_selector_display(&self.smallest_bin_selector),
            self.largest_bin,
            server_selector_display(&self.largest_bin_selector),
            self.nerr,
            self.external_servers.iter()
                .map(|(external_server, conn_result)| {
                    let conn_result = if *conn_result {"connected successfully"} else {"did not connect"};
                    [external_server, ": ", conn_result].concat()
                })
                .collect::<Vec<String>>()
                .join("\n\t\t"),
            self.invalid_references.iter()
                .map(|(server_details, invalid_reference, response_outcome)| {
                    let response_outcome = match *response_outcome {
                        ResponseOutcome::ConnectionFailed => "connection failed",
                        ResponseOutcome::FileTooLong => "file too long",
                        ResponseOutcome::Timeout => "response timed out",
                        ResponseOutcome::MissingEndLine => "missing end-line character",
                        _ => ""
                    };
                    [server_details, ": ", invalid_reference, " ", response_outcome].concat()
                })
                .collect::<Vec<String>>()
                .join("\n\t\t"),
        )
    }

    pub fn crawl(&mut self, selector: &str, server_name: &str, server_port: u16) -> std::io::Result<()> {
        let request = Request::new(selector, server_name, server_port, ItemType::DIR);
        
        // TODO: avoid clone
        self.used.push((server_name.to_string(), server_port, selector.to_string()));

        // TODO: Actually handle errors
        let response = gopher::send_and_recv(&request)
            .map_err(|error| {
                debug_eprintln!("Problem sending OR receving request: {error}");
                error
        })?;

        match response.response_outcome {
            ResponseOutcome::Complete => {
                for response_line in response.to_response_lines() {
                    if let Some(response_line) = response_line {
                        self.process_response_line(response_line).map_err(|error| {
                            debug_eprintln!("Problem processing response line: {error}");
                            error
                        })?;
                    } else {()} // Malformed request line (e.g. empty String)
                }
                self.dirs.push((request.server_details.clone(), selector.to_string()));
                self.ndir += 1;
            }
            _ => {
                self.invalid_references.push((request.server_details.clone(), selector.to_string(), response.response_outcome));
            }
        }
        
        Ok(())
    }

    fn process_response_line(&mut self, response_line: ResponseLine) -> std::io::Result<()> {    
        match response_line.item_type {
            ItemType::TXT => self.handle_supported_file(response_line, ItemType::TXT)?,
            ItemType::DIR => self.handle_dir(response_line)?,
            ItemType::ERR => self.nerr += 1,
            ItemType::BIN => self.handle_supported_file(response_line, ItemType::BIN)?,
            ItemType::DOT | ItemType::UNKNOWN => (), // TODO: Should we do anything else?
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
        let mut f = File::create(file_path).map_err(|error| {
            debug_eprintln!("Unable to create new file: {error}");
            error
        })?;
        f.write_all(buffer)?;
        Ok(f)
    }

    fn handle_dir(&mut self, response_line: ResponseLine) -> std::io::Result<()> {
        if response_line.server_name != SERVER_NAME {
            // TODO: Handle external servers
            // Only need to try connecting
            // Get the current local time
            #[allow(unused_variables)]
            let local_time = Local::now();
            // TODO: Use format everywhere
            match gopher::connect(&format!("{}:{}", response_line.server_name, response_line.server_port)) {
                Ok(_) => {
                    debug_println!("[{:02}h:{:02}m:{:02}s]: CONNECTED TO EXTERNAL {} ON {}", 
                        local_time.time().hour(), local_time.time().minute(), local_time.time().second(),
                        response_line.server_name, response_line.server_port);
                    self.external_servers.push((response_line.server_name.to_string(), true));
                    return Ok(())
                },
                Err(_) => {
                    debug_println!("[{:02}h:{:02}m:{:02}s]: FAILED TO CONNECT TO EXTERNAL {} ON {}", 
                        local_time.time().hour(), local_time.time().minute(), local_time.time().second(),
                        response_line.server_name, response_line.server_port);
                    self.external_servers.push((response_line.server_name.to_string(), false));
                    return Ok(())
                },
            }
        }

        if self.has_crawled(response_line.server_name, response_line.server_port.parse().unwrap(), response_line.selector) { return Ok(()) }
        
        self.crawl(response_line.selector, 
            response_line.server_name, 
            response_line.server_port.parse().unwrap()
        )?;
        Ok(())
    }

    fn handle_supported_file(&mut self, response_line: ResponseLine, file_type: ItemType) -> std::io::Result<()> {
        if self.has_crawled(response_line.server_name, response_line.server_port.parse().unwrap(), response_line.selector) { return Ok(()) }
        
        self.used.push((response_line.server_name.to_string(), response_line.server_port.parse().unwrap(), response_line.selector.to_string()));
        
        #[allow(unused_variables)]
        let file_type_display = match file_type {
            ItemType::TXT => "TXT",
            ItemType::BIN => "BIN",
            _ => "UNSUPPORTED FILE",
        };

        let request = Request::new(
            response_line.selector, 
            response_line.server_name, 
            response_line.server_port.parse().unwrap(),
            file_type,
        );

        let response = gopher::send_and_recv(&request).map_err(|error| {
            debug_eprintln!("Error sending or receving {file_type_display} file: {error}");
            error
        })?;

        match response.response_outcome {
            ResponseOutcome::Complete => {
                let f = Crawler::download_file(response_line.selector, &response.buffer).map_err(|error| {
                    debug_eprintln!("Error downloading {file_type_display} file: {error}");
                    error
                })?;
                match f.metadata() {
                    Ok(metadata) => {
                        let file_size = metadata.len();
                        match request.item_type {
                            ItemType::TXT => {
                                if file_size > self.largest_txt {
                                    self.largest_txt = file_size;
                                    // TODO: Can we use references instead?
                                    self.largest_txt_selector = (request.server_details.clone(), response_line.selector.to_string());
                                }
                                if file_size < self.smallest_txt {
                                    self.smallest_txt = file_size;
                                    self.smallest_txt_selector = (request.server_details.clone(), response_line.selector.to_string());
                                    // TODO: Use the file instead?? might not be worth
                                    self.smallest_contents = str::from_utf8(&response.buffer).expect("Ivalid UTF-8 sequence").to_string();  // TODO: Handle error??f.bytes().
                                }
                            },
                            ItemType::BIN => {
                                if file_size > self.largest_bin {
                                    self.largest_bin = file_size;
                                    self.largest_bin_selector = (request.server_details.clone(), response_line.selector.to_string());
                                }
                                if file_size < self.smallest_bin {
                                    self.smallest_bin = file_size;
                                    self.smallest_bin_selector = (request.server_details.clone(), response_line.selector.to_string());
                                }
                            },
                            _ => (),
                        }
                    },
                    Err(error) => {
                        debug_eprintln!("Error accessing {file_type_display} file metadata: {error}");
                        return Err(error)
                    },
                }
                match request.item_type {
                    ItemType::TXT => {
                        self.ntxt += 1;
                        self.txt_files.push((request.server_details.clone(), response_line.selector.to_string())); // TODO: Can we use references instead?
                    },
                    ItemType::BIN => {
                        self.nbin += 1;
                        self.bin_files.push((request.server_details.clone(), response_line.selector.to_string())); // TODO: Can we use references instead of clone and to_string?
                    },
                    _ => (),
                }
            }
            _ => {
                self.invalid_references.push((request.server_details.clone(), response_line.selector.to_string(), response.response_outcome));
            }
        }
        Ok(())
    }

    fn has_crawled(&self, server_details: &str, server_port: u16, selector: &str) -> bool {
        self.used.iter()
            .any(|(used_server_details, used_server_port, used_selector)| {
                used_server_details == server_details && 
                *used_server_port == server_port &&
                used_selector == selector
        })
    }
}