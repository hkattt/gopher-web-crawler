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

// TODO: Can we use references instread?
pub struct Crawler {
    root_server_name: Rc<String>,
    root_server_port: u16,

    ndir: u32,                                           // The number of directories
    dirs: Vec<(Rc<String>, Rc<String>)>,                         // List of all directories (server details, directory)

    ntxt:  u32,                                          // The number of simple text files
    txt_files: Vec<(Rc<String>, Rc<String>)>,                    // List of all simple text tiles (full path) (server details, text file)
    
    nbin:  u32,                                          // The number of binary (i.e. non-text) files
    bin_files: Vec<(Rc<String>, Rc<String>)>,                    // List of all binary files (full path) (server details, binary file)
    
    smallest_contents: String,                           // Contents of the smallest text file
    smallest_txt: u64,                                   // The size of the smallest text file
    largest_txt: u64,                                    // The size of the largest text file
    
    smallest_bin: u64,                                   // The size of the smallest binary file
    largest_bin: u64,                                    // The size of the largest binary file
    
    smallest_txt_selector: (Rc<String>, Rc<String>),             // The selector of the smallest text file
    largest_txt_selector:  (Rc<String>, Rc<String>),              // The selector of the largest text file
    smallest_bin_selector: (Rc<String>, Rc<String>),             // The selector of the smallest binary file
    largest_bin_selector:  (Rc<String>, Rc<String>),              // The selector of the largest binary file
    
    nerr: u32,                                           // The number of unique invalid references (error types)
    external_servers: Vec<(Rc<String>, u16, bool)>,               // TODO: Fix List of external servers and if they accepted a connection
    invalid_references: Vec<(String, ResponseOutcome)>,  // List of references that have "issues/errors" that had be explicitly dealt with
    used: Vec<(Rc<String>, u16, Rc<String>)>,                         // Used (server name, server port, selector)
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
    pub fn new(server_name: String, server_port: u16) -> Crawler {
        Crawler { root_server_name: server_name.into(), root_server_port: server_port, ..Default::default() }
    }

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

    pub fn start_crawl(&mut self, starting_selector: String) -> std::io::Result<()> {
        // TODO: Can we specify a starting selector?
        // TODO: Can we fix this without cloning?
        // self.crawl(STARTING_SELECTOR, self.root_server_name, self.root_server_port)?;
        // TODO: How use root server name!!
        // TODO: Do these need to be Rcs?
        self.crawl(
            Rc::new(starting_selector), 
            Rc::clone(&self.root_server_name), 
            self.root_server_port
        )?;

        Ok(())
    }

    fn crawl(&mut self, selector: Rc<String>, server_name: Rc<String>, server_port: u16) -> std::io::Result<()> {
        self.used.push((
            Rc::clone(&server_name), 
            server_port, 
            Rc::clone(&selector)
        ));

        let request = Request::new(
            Rc::clone(&selector), 
            Rc::clone(&server_name), 
            server_port, 
            ItemType::Dir
        );
        
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
        // External server
        // External server is anything with a different server name OR a different port 
        if response_line.server_name != self.root_server_name || response_line.server_port != self.root_server_port {
            // Get the current local time
            #[allow(unused_variables)]
            let local_time = Local::now();
            // TODO: Use format everywhere
            match gopher::connect(&format!("{}:{}", response_line.server_name, response_line.server_port)) {
                Ok(_) => {
                    debug_println!("[{:02}h:{:02}m:{:02}s]: CONNECTED TO EXTERNAL {} ON {}", 
                        local_time.time().hour(), local_time.time().minute(), local_time.time().second(),
                        response_line.server_name, response_line.server_port);
                    self.external_servers.push((response_line.server_name, response_line.server_port, true));
                    return Ok(())
                },
                Err(_) => {
                    debug_println!("[{:02}h:{:02}m:{:02}s]: FAILED TO CONNECT TO EXTERNAL {} ON {}", 
                        local_time.time().hour(), local_time.time().minute(), local_time.time().second(),
                        response_line.server_name, response_line.server_port);
                    // TODO: Should we just pass string server_port?
                    self.external_servers.push((response_line.server_name, response_line.server_port, false));
                    return Ok(())
                },
            }
        }

        if self.has_crawled(&response_line.server_name, response_line.server_port, &response_line.selector) { return Ok(()) }
        
        // TODO: Is it good to use reference here?
        self.crawl(response_line.selector, 
            response_line.server_name, 
            response_line.server_port
        )?;
        Ok(())
    }

    fn handle_file(&mut self, response_line: ResponseLine, file_type: ItemType) -> std::io::Result<()> {
        // TODO: Is there better syntax?
        if self.has_crawled(&*response_line.server_name, response_line.server_port, &*response_line.selector) { return Ok(()) }
        
        self.used.push((
            Rc::clone(&response_line.server_name), 
            response_line.server_port,
            Rc::clone(&response_line.selector))
        );
        
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
                ); // TODO: Can we use references instead?

                if file_size > self.largest_txt {
                    self.largest_txt = file_size;
                    // TODO: Can we use references instead?
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
                    // TODO: Use the file instead?? might not be worth
                    self.smallest_contents = str::from_utf8(buffer).expect("Ivalid UTF-8 sequence").to_string();  // TODO: Handle error??f.bytes().
                }
            },
            ItemType::Bin => {
                self.nbin += 1;
                self.bin_files.push((
                    Rc::clone(&request.server_details), 
                    Rc::clone(&request.selector)
                )); // TODO: Can we use references instead of clone and to_string?

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