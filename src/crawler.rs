use std::{
    cmp::min, 
    fs::{File, Metadata}, 
    io::Write, 
    str,
    rc::Rc
};

// Chrono imports for data-time functionality
use chrono::Local;
#[allow(unused_imports)]
use chrono::Timelike;

use::debug_print::{debug_println, debug_eprintln};

use crate::gopher::{
    self, 
    request::Request, 
    response::{ItemType, ResponseLine, ResponseLineError, ResponseOutcome}
};

use crate::{MAX_FILENAME_LEN, OUTPUT_FOLDER};

/// Represents a Gopher server craweler. 
/// 
/// * `root_server_name`: Hostname of the root (start) server
/// * `root_server_port`: Port number of the root (start) server
/// 
/// * `ndir`: Number of directories
/// * `dirs`: List of all directories (server details, directory) pairs
/// 
/// * `ntxt`: Number of text files
/// * `txt_files`: List of all simple text files (server details, text file) pairs
/// 
/// * `nbin`: Number of binary files
/// * `bin_files`: List of all binary files (server details, binary file) pairs
/// 
/// * `smallest_contets`: Contents of the smallest text file
/// * `smallest_txt`: Size of the smallest text file (bytes)
/// * `largest_txt`: Size of the largest text file (bytes)
/// 
/// * `smallest_bin`: Size of the smallest binary file
/// * `largest_bin`: Size of the largest binary file
/// 
/// * `smallest_txt_selector`: The selector of the smallest text file 
/// (server details, text file selector) pairs
/// * `largest_txt_selector`: The selector of the largest text file 
/// (server details, text file selector) pairs
/// * `smallest_bin_selector`: The selector of the smallest binary file 
/// (server details, binary file selector) pairs
/// * `largest_bin_selector`: The selector of the largest binary file 
/// (server details, binary file selector) pairs
/// 
/// * `nerr`: The number of unique invalid references (error types)
/// * `external_references`: List of external servers and if they accepted
/// a connection (server name, server port, connected) triples
/// * `invalid_references`: List of invalid references 
/// (details of the request, response outcome) pairs
/// * `used`: List of used selectors (server name, server port, selector) tripless
pub struct Crawler {
    root_server_name: Rc<String>,
    root_server_port: u16,

    ndir: u32,
    dirs: Vec<(Rc<String>, Rc<String>)>,

    ntxt: u32,
    txt_files: Vec<(Rc<String>, Rc<String>)>,
    
    nbin: u32,
    bin_files: Vec<(Rc<String>, Rc<String>)>,
    
    smallest_contents: String,
    smallest_txt: u64,
    largest_txt: u64,
    
    smallest_bin: u64,
    largest_bin: u64,
    
    smallest_txt_selector: (Rc<String>, Rc<String>),
    largest_txt_selector:  (Rc<String>, Rc<String>),
    smallest_bin_selector: (Rc<String>, Rc<String>),
    largest_bin_selector:  (Rc<String>, Rc<String>),
    
    nerr: u32,
    external_servers: Vec<(Rc<String>, u16, bool)>,
    invalid_references: Vec<(String, ResponseOutcome)>,
    used: Vec<(Rc<String>, u16, Rc<String>)>,
}

impl Default for Crawler {
    fn default() -> Crawler{
        Crawler {
            root_server_name: Rc::new(String::from("comp3310.ddns.net")),
            root_server_port: 70,

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
            
            smallest_txt_selector: (Rc::new(String::new()), Rc::new(String::new())),
            largest_txt_selector:  (Rc::new(String::new()), Rc::new(String::new())),
            smallest_bin_selector: (Rc::new(String::new()), Rc::new(String::new())),
            largest_bin_selector:  (Rc::new(String::new()), Rc::new(String::new())),
            
            nerr: 0,
            external_servers: Vec::new(),
            invalid_references: Vec::new(),
            used: Vec::new(),
        }
    }
}

impl Crawler {
    /// Constructs a new `Crawler` instance with the given optional parameters.
    /// 
    /// If `server_name` is provided, it will be used as the root server name. Otherwise,
    /// the default root server name: comp3310.ddns.net is used.
    /// 
    /// If `server_port` is provided, it will be used as the root server port. Otherwise,
    /// the default root server port: 70 is used.
    /// 
    /// # Arguments
    /// 
    /// * `server_name`: Optional parameter specifying the root server name.
    /// * `server_port`: Optional parameter specifying the root server port.
    /// 
    /// # Returns 
    /// 
    /// A new `Crawler` instance with the specified or default parameters.
    pub fn new(server_name: Option<String>, server_port: Option<u16>) -> Crawler {
        Crawler { 
            // Use `server_name` if it is provided. Otherwise, use the default.
            root_server_name: server_name.map_or_else(
                || Crawler::default().root_server_name, 
                Rc::new
            ),
            // Use `server_port` if it is provided. Otherwise, use the default.
            root_server_port: server_port.unwrap_or(
                Crawler::default().root_server_port
            ),
            ..Default::default() 
        }
    }

    /// Reports the outcome of a server crawl
    pub fn report(&self) {
        let format_server_selector = |(server_details, selector): &(Rc<String>, Rc<String>)| {
            format!("{}: {}", *server_details, *selector)
        };
    
        let format_external_server = |(server_name, server_port, conn_result): &(Rc<String>, u16, bool)| {
            let status = if *conn_result {
                "connected successfully"
            } else {
                "did not connect"
            };
            format!("{}:{} {}", *server_name, *server_port, status)
        };
    
        let format_invalid_reference = |(response_details, response_outcome): &(String, ResponseOutcome)| {
            format!("{} {}", response_outcome.to_string(), response_details)
        };

        let sort_alphabetically = |mut v: Vec<String>| {
            v.sort_by(|a, b| {
                a.to_lowercase().cmp(&b.to_lowercase())
            });
            v
        };

        let sorted_dirs = sort_alphabetically(
            self.dirs.iter().map(format_server_selector).collect::<Vec<String>>()
        );
        let sorted_txt_files = sort_alphabetically(
            self.txt_files.iter().map(format_server_selector).collect::<Vec<String>>()
        );
        let sorted_bin_files = sort_alphabetically(
            self.bin_files.iter().map(format_server_selector).collect::<Vec<String>>()
        );
        let sorted_external_servers = sort_alphabetically(
            self.external_servers.iter().map(format_external_server).collect::<Vec<_>>()
        );
        let sorted_invalid_references = sort_alphabetically(
            self.invalid_references.iter().map(format_invalid_reference).collect::<Vec<_>>()
        );

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
            sorted_dirs.join("\n\t\t"),
            self.ntxt,
            sorted_txt_files.join("\n\t\t"),
            self.nbin,
            sorted_bin_files.join("\n\t\t"),
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
            sorted_external_servers.join("\n\t\t"),
            sorted_invalid_references.join("\n\t\t"),
        );
    }

    /// Starts a Gopher server crawl on the root server name 
    /// and root server port
    pub fn start_crawl(&mut self) -> std::io::Result<()> {
        // Send an empty selector to start the call
        let starting_selector = String::from("");

        self.crawl(
            Rc::new(starting_selector), 
            Rc::clone(&self.root_server_name), 
            self.root_server_port
        )?;

        Ok(())
    }

    /// Crawls a given Gopher server with a selector
    /// 
    /// # Arguments
    /// * `selector`: Selector string being used to request an item
    /// * `server_name`: The name of the Gopher server to be crawled
    /// * `server_port`: The port number of the Gopher server to be crawled
    /// 
    /// # Returns
    /// Nothing if sucessfull. An IO error is unsucessful.
    fn crawl(&mut self, selector: Rc<String>, server_name: Rc<String>, server_port: u16) -> std::io::Result<()> {
        self.used.push((
            Rc::clone(&server_name), 
            server_port, 
            Rc::clone(&selector)
        ));

        // Request to send to the server
        let request = Request::new(
            Rc::clone(&selector), 
            Rc::clone(&server_name), 
            server_port, 
            ItemType::Dir
        );
        
        let response = gopher::send_and_recv(&request)
            .map_err(|error| {
                debug_eprintln!("Problem sending OR receving request: {error}");
                error
        })?;

        match response.response_outcome {
            ResponseOutcome::Complete => {
                // Split the response into response lines
                for response_line in response.to_response_lines() {
                    match response_line {
                        // Process the response line
                        Ok(response_line) => {
                            self.process_response_line(response_line).map_err(|error| {
                                debug_eprintln!("Problem processing response line: {error}");
                                error
                            })?;
                        },
                        // Invalid response line
                        Err(error) => {
                            match error {
                                ResponseLineError::Empty => (),
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
                self.dirs.push((request.server_details, selector));
                self.ndir += 1;
            }
            // Response unsucessful
            _ => {
                self.invalid_references.push((
                    format!("{} {}", request.server_details, selector),
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
        // External server is anything with a different server name OR a different port 
        if response_line.server_name != self.root_server_name || response_line.server_port != self.root_server_port {
            // Get the current local time
            #[allow(unused_variables)]
            let local_time = Local::now();

            // Attempts to connect to the external server
            match gopher::connect(&format!("{}:{}", response_line.server_name, response_line.server_port)) {
                // Connected sucessfully
                Ok(_) => {
                    debug_println!("[{:02}h:{:02}m:{:02}s]: CONNECTED TO EXTERNAL {} ON {}", 
                        local_time.time().hour(), local_time.time().minute(), local_time.time().second(),
                        response_line.server_name, response_line.server_port);

                    self.external_servers.push((response_line.server_name, response_line.server_port, true));
                    return Ok(())
                },
                // Failed to connect
                Err(_) => {
                    debug_println!("[{:02}h:{:02}m:{:02}s]: FAILED TO CONNECT TO EXTERNAL {} ON {}", 
                        local_time.time().hour(), local_time.time().minute(), local_time.time().second(),
                        response_line.server_name, response_line.server_port);

                    self.external_servers.push((response_line.server_name, response_line.server_port, false));
                    return Ok(())
                },
            }
        }

        // Check if the directory has been crawled before
        if self.has_crawled(&response_line.server_name, response_line.server_port, &response_line.selector) { 
            return Ok(()) 
        }
        
        // Crawl the directory
        self.crawl(response_line.selector, 
            response_line.server_name, 
            response_line.server_port
        )?;
        Ok(())
    }

    fn handle_file(&mut self, response_line: ResponseLine, file_type: ItemType) -> std::io::Result<()> {
        // Check if the file has been crawled before
        if self.has_crawled(&*response_line.server_name, response_line.server_port, &*response_line.selector) { 
            return Ok(()) 
        }
        
        self.used.push((
            Rc::clone(&response_line.server_name), 
            response_line.server_port,
            Rc::clone(&response_line.selector))
        );
        
        // Request to send to the server
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
            // Sucessful transaction
            ResponseOutcome::Complete => {
                let f = Crawler::download_file(&request.selector, &response.buffer).map_err(|error| {
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
            // Unsucessful transaction
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
                self.txt_files.push((
                    Rc::clone(&request.server_details), 
                    Rc::clone(&request.selector))
                ); 

                if file_size > self.largest_txt {
                    self.largest_txt = file_size;
                    self.largest_txt_selector = (
                        Rc::clone(&request.server_details), 
                        Rc::clone(&request.selector)
                    );
                }

                if file_size < self.smallest_txt {
                    self.smallest_txt = file_size;
                    self.smallest_txt_selector = (
                        Rc::clone(&request.server_details), 
                        Rc::clone(&request.selector)
                    );
                    self.smallest_contents = str::from_utf8(buffer).expect("Ivalid UTF-8 sequence").to_string();  
                }
            },
            ItemType::Bin => {
                self.nbin += 1;
                self.bin_files.push((
                    Rc::clone(&request.server_details), 
                    Rc::clone(&request.selector)
                )); 

                if file_size > self.largest_bin {
                    self.largest_bin = file_size;
                    self.largest_bin_selector = (
                        Rc::clone(&request.server_details), 
                        Rc::clone(&request.selector)
                    );
                }

                if file_size < self.smallest_bin {
                    self.smallest_bin = file_size;
                    self.smallest_bin_selector = (
                        Rc::clone(&request.server_details), 
                        Rc::clone(&request.selector)
                    );
                }
            },
            _ => (),
        }
    }

    fn has_crawled(&self, server_details: &str, server_port: u16, selector: &str) -> bool {
        self.used.iter()
            .any(|(used_server_details, used_server_port, used_selector)| {
                **used_server_details == server_details && 
                *used_server_port == server_port &&
                **used_selector == selector
        })
    }

    fn download_file(selector: &str, buffer: &[u8]) -> std::io::Result<File> {
        // Remove the / prefix from the selector. Truncate long selector names
        let file_name = &selector[1..min(selector.len(), MAX_FILENAME_LEN + 1)];

        // Replace forward slashes with dashes to create a valid file name
        let file_name = file_name.replace("/", "-");
        
        let file_path = format!("{}/{}", OUTPUT_FOLDER, &file_name);
        let mut f = File::create(file_path).map_err(|error| {
            debug_eprintln!("Unable to create new file: {error}");
            error
        })?;
        f.write_all(buffer)?;
        
        Ok(f)
    }
}