mod crawler;
mod gopher;

use std::{
    env, 
    fs::{self, remove_dir_all}, 
    io::ErrorKind, 
    path::Path
};

use crawler::Crawler;

const CRLF: &str              = "\r\n";
const TAB: &str               = "\t";
const OUTPUT_FOLDER: &str     = "out";
const MAX_CHUNK_SIZE: usize   = 4096; 
const MAX_FILENAME_LEN: usize = 255;  

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut server_name = None;
    let mut server_port = None;
    let mut remove_dirs = true;

    let mut args_iter = env::args().skip(1);
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            // Server name argument
            "-n" => {
                server_name = Some(
                    args_iter.next().ok_or("Missing server name after -n")?
                );
            }
            // Server port argument
            "-p" => {
                let port_str = args_iter.next().ok_or("Missing server port after -p")?;
                server_port = match port_str.parse() {
                    Ok(port) => Some(port),
                    Err(_) => {
                        eprintln!("Server port must be an integer");
                        return Ok(())
                    }
                };
            }
            // Directory delete argument
            "-d" => {
                remove_dirs = false;
            }
            // Invalid argument
            _ => {
                eprintln!("Usage: gopher [-n <server_name>] [-p <server_port>] [-d]");
                return Ok(())
            }
        }
    }

    // Create output directory to store files
    if let Err(error) = fs::create_dir(Path::new(&OUTPUT_FOLDER)) {
        if error.kind() != ErrorKind::AlreadyExists {
            panic!("Unable to create output folder: {error}");
        }
    }

    // Crawl the Gopher server and report the results
    let mut crawler = Crawler::new(server_name, server_port);
    crawler.start_crawl()?;
    crawler.report();

    // Remove output directory and all of its contents
    if remove_dirs {
        remove_dir_all(OUTPUT_FOLDER)?;
    }

    Ok(())
}