use crawler::Crawler;

mod crawler;

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
const MAX_FILENAME_LEN: usize = 255; // TODO: Can we get this from the OS somehow??

fn main() -> std::io::Result<()> {
    // TODO: Should we make SERVER_PORT a &str?
    let mut crawler = Crawler::new();
    // TODO: Create this with builder?
    crawler.crawl(STARTING_SELECTOR, SERVER_NAME, SERVER_PORT)?;
    Ok(())
}