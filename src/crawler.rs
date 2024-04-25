use std::{
    cmp::min, 
    fs::{File, Metadata}, 
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
    response::{ItemType, ResponseLine, ResponseOutcome, ResponseLineError}
};

use crate::{MAX_FILENAME_LEN, OUTPUT_FOLDER};

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
    external_servers: Vec<(String, u16, bool)>,               // TODO: Fix List of external servers and if they accepted a connection
    invalid_references: Vec<(String, ResponseOutcome)>,  // List of references that have "issues/errors" that had be explicitly dealt with
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
        let format_server_selector = |(server_details, selector): &(String, String)| {
            format!("{server_details}: {selector}")
        };
    
        let format_external_server = |(server_name, server_port, conn_result): &(String, u16, bool)| {
            let status = if *conn_result {
                "connected successfully"
            } else {
                "did not connect"
            };
            format!("{}:{} {}", server_name, *server_port, status)
        };
    
        let format_invalid_reference = |(response_details, response_outcome): &(String, ResponseOutcome)| {
            format!("{} {}", response_outcome.to_string(), response_details)
        };

        println!(
            "\nSTART CRAWLER REPORT\n\n\
            \tNumber of Gopher directories: {}\n\
            \t\t{}\n\n\
            \tNumber of simple text files: {}\n\
            \t\t{}\n\n\
            \tNumber of binary files: {}\n\
            \t\t{}\n\n\
            \tSmallest text file: {}\n\
            \t\tSize: {} bytes\n\
            \t\tContents: {}\n\n\
            \tSize of the largest text file: {} bytes\n\
            \t\t{}\n\n\
            \tSize of the smallest binary file: {} bytes\n\
            \t\t{}\n\n\
            \tSize of the largest binary file: {} bytes\n\
            \t\t{}\n\n\
            \tThe number of unique invalid references (error types): {}\n\n\
            \tList of external servers:\n\
            \t\t{}\n\n\
            \tReferences that have issues/errors:\n\
            \t\t{}\n\n\
            END CRAWLER REPORT",
            self.ndir,
            self.dirs.iter().map(format_server_selector).collect::<Vec<_>>().join("\n\t\t"),
            self.ntxt,
            self.txt_files.iter().map(format_server_selector).collect::<Vec<_>>().join("\n\t\t"),
            self.nbin,
            self.bin_files.iter().map(format_server_selector).collect::<Vec<_>>().join("\n\t\t"),
            format_server_selector(&self.smallest_txt_selector),
            self.smallest_txt,
            self.smallest_contents,
            self.largest_txt,
            format_server_selector(&self.largest_txt_selector),
            self.smallest_bin,
            format_server_selector(&self.smallest_bin_selector),
            self.largest_bin,
            format_server_selector(&self.largest_bin_selector),
            self.nerr,
            self.external_servers.iter().map(format_external_server).collect::<Vec<_>>().join("\n\t\t"),
            self.invalid_references.iter().map(format_invalid_reference).collect::<Vec<_>>().join("\n\t\t"),
        );
    }

    pub fn crawl(&mut self, selector: &str, server_name: &str, server_port: u16) -> std::io::Result<()> {
        let request = Request::new(selector, server_name, server_port, ItemType::Dir);
        
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
                    match response_line {
                        Ok(response_line) => {
                            self.process_response_line(response_line).map_err(|error| {
                                debug_eprintln!("Problem processing response line: {error}");
                                error
                            })?;
                        },
                        Err(error) => {
                            match error {
                                ResponseLineError::Empty => (),
                                // TODO: Fix this man
                                ResponseLineError::InvalidParts(line) => {
                                    self.invalid_references.push((line, ResponseOutcome::MalformedResponseLine));
                                },
                                ResponseLineError::EmptyDisplayString(line) => {
                                    self.invalid_references.push((line, ResponseOutcome::MalformedResponseLine));
                                },
                                ResponseLineError::EmptyHost(server_name, server_port, selector) => {
                                    self.invalid_references.push((
                                        format!("{}:{} {}", server_name, server_port, selector),
                                        ResponseOutcome::MalformedResponseLine
                                    ));
                                }, 
                                ResponseLineError::NonIntPort(server_name, server_port, selector) => {
                                    self.invalid_references.push((
                                        format!("{}:{} {}", server_name, server_port, selector), 
                                        ResponseOutcome::MalformedResponseLine
                                    ));
                                },
                            }
                        }
                    }
                }
                self.dirs.push((request.server_details.clone(), selector.to_string()));
                self.ndir += 1;
            }
            _ => {
                self.invalid_references.push((
                    format!("{} {}", request.server_details.clone(), selector.to_string()),
                    response.response_outcome
                ));
            }
        }
        Ok(())
    }

    fn process_response_line(&mut self, response_line: ResponseLine) -> std::io::Result<()> {    
        match response_line.item_type {
            ItemType::Txt => self.handle_file(response_line, ItemType::Txt)?,
            ItemType::Dir => self.handle_dir(response_line)?,
            ItemType::Err => self.nerr += 1,
            ItemType::Bin => self.handle_file(response_line, ItemType::Bin)?,
            ItemType::Unknown => (), 
        }
        Ok(())
    }

    fn handle_dir(&mut self, response_line: ResponseLine) -> std::io::Result<()> {
        // External server
        // External server is anything with a different server name OR a different port 
        if response_line.server_name != self.root_server_name() || response_line.server_port != self.root_server_port() {
            // Get the current local time
            #[allow(unused_variables)]
            let local_time = Local::now();
            // TODO: Use format everywhere
            match gopher::connect(&format!("{}:{}", response_line.server_name, response_line.server_port)) {
                Ok(_) => {
                    debug_println!("[{:02}h:{:02}m:{:02}s]: CONNECTED TO EXTERNAL {} ON {}", 
                        local_time.time().hour(), local_time.time().minute(), local_time.time().second(),
                        response_line.server_name, response_line.server_port);
                    self.external_servers.push((response_line.server_name.to_string(), response_line.server_port, true));
                    return Ok(())
                },
                Err(_) => {
                    debug_println!("[{:02}h:{:02}m:{:02}s]: FAILED TO CONNECT TO EXTERNAL {} ON {}", 
                        local_time.time().hour(), local_time.time().minute(), local_time.time().second(),
                        response_line.server_name, response_line.server_port);
                    // TODO: Should we just pass string server_port?
                    self.external_servers.push((response_line.server_name.to_string(), response_line.server_port, false));
                    return Ok(())
                },
            }
        }

        if self.has_crawled(response_line.server_name, response_line.server_port, response_line.selector) { return Ok(()) }
        
        self.crawl(response_line.selector, 
            response_line.server_name, 
            response_line.server_port
        )?;
        Ok(())
    }

    fn handle_file(&mut self, response_line: ResponseLine, file_type: ItemType) -> std::io::Result<()> {
        if self.has_crawled(response_line.server_name, response_line.server_port, response_line.selector) { return Ok(()) }
        
        self.used.push((response_line.server_name.to_string(), response_line.server_port, response_line.selector.to_string()));
        
        let request = Request::new(
            response_line.selector, 
            response_line.server_name, 
            response_line.server_port,
            file_type,
        );

        let response = gopher::send_and_recv(&request).map_err(|error| {
            debug_eprintln!("Error sending or receving {} file: {}", request.item_type.to_string(), error);
            error
        })?;

        match response.response_outcome {
            ResponseOutcome::Complete => {
                let f = Crawler::download_file(response_line.selector, &response.buffer).map_err(|error| {
                    debug_eprintln!("Error downloading {} file: {}", request.item_type.to_string(), error);
                    error
                })?;
                match f.metadata() {
                    Ok(metadata) => self.update_file_stats(metadata, &request, &response.buffer),
                    Err(error) => {
                        debug_eprintln!("Error accessing {} file metadata: {}", request.item_type.to_string(), error);
                        return Err(error)
                    }
                }
            }
            _ => {
                self.invalid_references.push((
                    format!("{} {}", request.server_details.clone(), request.selector.to_string()),
                    response.response_outcome
                ));
            }
        }
        Ok(())
    }

    fn update_file_stats(&mut self, file_metadata: Metadata, request: &Request, buffer: &Vec<u8>) {
        let file_size = file_metadata.len();
        match request.item_type {
            ItemType::Txt => {
                self.ntxt += 1;
                self.txt_files.push((request.server_details.clone(), request.selector.to_string())); // TODO: Can we use references instead?

                if file_size > self.largest_txt {
                    self.largest_txt = file_size;
                    // TODO: Can we use references instead?
                    self.largest_txt_selector = (request.server_details.clone(), request.selector.to_string());
                }

                if file_size < self.smallest_txt {
                    self.smallest_txt = file_size;
                    self.smallest_txt_selector = (request.server_details.clone(), request.selector.to_string());
                    // TODO: Use the file instead?? might not be worth
                    self.smallest_contents = str::from_utf8(buffer).expect("Ivalid UTF-8 sequence").to_string();  // TODO: Handle error??f.bytes().
                }
            },
            ItemType::Bin => {
                self.nbin += 1;
                self.bin_files.push((request.server_details.clone(), request.selector.to_string())); // TODO: Can we use references instead of clone and to_string?

                if file_size > self.largest_bin {
                    self.largest_bin = file_size;
                    self.largest_bin_selector = (request.server_details.clone(), request.selector.to_string());
                }

                if file_size < self.smallest_bin {
                    self.smallest_bin = file_size;
                    self.smallest_bin_selector = (request.server_details.clone(), request.selector.to_string());
                }
            },
            _ => (),
        }
    }

    fn has_crawled(&self, server_details: &str, server_port: u16, selector: &str) -> bool {
        self.used.iter()
            .any(|(used_server_details, used_server_port, used_selector)| {
                used_server_details == server_details && 
                *used_server_port == server_port &&
                used_selector == selector
        })
    }

    fn root_server_name(&self) -> &String {
        &self.used[0].0
    }

    fn root_server_port(&self) -> u16 {
        self.used[0].1
    }

    // TODO: Should this use self or???
    fn download_file(selector: &str, buffer: &[u8]) -> std::io::Result<File> {
        // Remove the / prefix from the selector. Truncate long selector names
        let file_name = &selector[1..min(selector.len(), MAX_FILENAME_LEN + 1)];
        // Replace forward slashes with dashes to create a valid file name
        let file_name = file_name.replace("/", "-");
        // TODO: Replace the string stuff with global variables?
        let file_path = format!("{}/{}", OUTPUT_FOLDER, &file_name);
        let mut f = File::create(file_path).map_err(|error| {
            debug_eprintln!("Unable to create new file: {error}");
            error
        })?;
        f.write_all(buffer)?;
        Ok(f)
    }
}