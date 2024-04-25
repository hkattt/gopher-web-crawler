mod crawler;
mod gopher;

use std::{
    fs::{self, remove_dir_all}, 
    io::ErrorKind, 
    path::Path
};

use crawler::Crawler;

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
const OUTPUT_FOLDER: &str     = "out";
const MAX_CHUNK_SIZE: usize   = 4096; 
const MAX_FILENAME_LEN: usize = 255;  // TODO: Can we get this from the OS somehow??

// TODO: What is up with invalid 0
// TODO: What about malformed1
// TODO: Debug mode
// TODO: Command line arguments: debug, server, remove directory

fn main() -> std::io::Result<()> {
    // Create output directory to store files
    if let Err(error) = fs::create_dir(Path::new(&OUTPUT_FOLDER)) {
        if error.kind() != ErrorKind::AlreadyExists {
            panic!("Unable to create output folder: {error}");
        }
    }
    // TODO: Should we make SERVER_PORT a &str?
    let mut crawler = Crawler::new();
    // TODO: Create this with builder?
    crawler.crawl(STARTING_SELECTOR, SERVER_NAME, SERVER_PORT)?;
    crawler.report();

    // Remove output directory and all of its contents
    remove_dir_all(OUTPUT_FOLDER)?;

    Ok(())
}