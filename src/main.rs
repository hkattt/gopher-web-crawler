use std::io::{Read, Write};
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    // Open server on Gophie with: comp3310.ddns.net:70

    //let mut stream = TcpStream::connect("127.0.0.1:70")?;

    let mut stream = TcpStream::connect("comp3310.ddns.net:70")?;

    let selector = String::from("\r\n");
    let mut buffer = String::new();

    stream.write(selector.as_bytes())?;
    stream.read_to_string(&mut buffer)?;

    println!("{buffer}");
    
    Ok(())
}
