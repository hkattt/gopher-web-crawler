use std::str;

use crate::{CRLF, TAB};

pub enum ResponseOutcome {
    Complete,
    Timeout,
    FileTooLong,
    ConnectionFailed,
    MissingEndLine,
    MalformedResponseLine,
}

impl ToString for ResponseOutcome {
    fn to_string(&self) -> String {
        match self {
            ResponseOutcome::Complete              => String::from("Completed sucessfully"),
            ResponseOutcome::Timeout               => String::from("Connection timed out"),
            ResponseOutcome::FileTooLong           => String::from("File too long"),
            ResponseOutcome::ConnectionFailed      => String::from("Failed to connect"),
            ResponseOutcome::MissingEndLine        => String::from("Missing end-line"),
            ResponseOutcome::MalformedResponseLine => String::from("Malformed response line"),
        }
    }
}

pub struct Response {
    pub buffer: Vec<u8>,    // Bytes received
    pub response_outcome: ResponseOutcome,        // TODO: Explain
}

impl Response {
    pub fn new(buffer: Vec<u8>, response_outcome: ResponseOutcome) -> Response {
        Response {
            buffer,
            response_outcome,
        }
    }

    pub fn to_response_lines<'a>(&'a self) -> Vec<Result<ResponseLine<'a>, ResponseLineError>> {
        // Convert byte stream into a string (i.e. UTF-8 sequence)
        let buffer = str::from_utf8(&self.buffer).expect("Ivalid UTF-8 sequence"); // TODO: Handle error??
        buffer.split(CRLF).map(|line| ResponseLine::new(line)).collect()
    }
}

pub enum ItemType {
    Txt,     // 0   Item is a text file
    Dir,     // 1   Item is a directory 
    Err,     // 3   Item is a error
    Bin,     // 9   Item is a binary file
    Unknown, // _   Item is unknown     
}

impl ToString for ItemType {
    fn to_string(&self) -> String {
        match self {
            ItemType::Txt     => String::from("TXT"),
            ItemType::Dir     => String::from("DIR"),
            ItemType::Err     => String::from("ERR"),
            ItemType::Bin     => String::from("BIN"),
            ItemType::Unknown => String::from("UNKNOWN"),
        }
    }
}

#[derive(Debug)]
pub enum ResponseLineError {
    Empty,
    InvalidParts(String),
    EmptyDisplayString(String),
    EmptyHost(String, String, String), 
    NonIntPort(String, String, String)
}

impl std::fmt::Display for ResponseLineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseLineError::Empty => write!(f, "Empty response line"),
            ResponseLineError::InvalidParts(line) => write!(f, "Unable to split line: {line}"),
            ResponseLineError::EmptyDisplayString(line) => write!(f, "Empty display string: {line}"),
            ResponseLineError::EmptyHost(_, _, _) => write!(f, "Missing host name"),
            ResponseLineError::NonIntPort(_, _, _) => write!(f, "Invalid port number"),
        }
    }
}

impl std::error::Error for ResponseLineError {}

pub struct ResponseLine<'a> {
    pub item_type:   ItemType,
    pub selector:    &'a str, 
    pub server_name: &'a str,
    pub server_port: u16,
}

impl<'a> ResponseLine<'a> {
    pub fn new(line: &'a str) -> Result<ResponseLine<'a>, ResponseLineError> {
        if line.is_empty() {
            return Err(ResponseLineError::Empty);
        }

        let mut parts = line.splitn(4, TAB);

        // TODO: Can we do this without cloning?
        if parts.clone().count() != 4 {
            return Err(ResponseLineError::InvalidParts(line.to_string()));
        }

        let user_display_string = parts.next().unwrap();

        let item_type = match user_display_string.chars().next() {
            Some(i) => match i {
                '0' => ItemType::Txt,
                '1' => ItemType::Dir,
                '3' => ItemType::Err,
                '9' => ItemType::Bin,
                _   => ItemType::Unknown
            },
            None => return Err(ResponseLineError::EmptyDisplayString(line.to_string()))
        };
        // Any selector is fine
        let selector = parts.next().unwrap();
        // TODO: Server name cannot be empty
        let server_name = parts.next().unwrap();
        if server_name.is_empty() {
            return Err(ResponseLineError::EmptyHost(server_name.to_string(), parts.next().unwrap().to_string(), selector.to_string()))
        }
        // TODO: Server port must be an integer
        let server_port_str = parts.next().unwrap();
        let server_port = server_port_str.parse::<u16>();
        let server_port = match server_port {
            Ok(port) => port,
            Err(_) => return Err(ResponseLineError::NonIntPort(server_name.to_string(), server_port_str.to_string(), selector.to_string())), 
        };
        Ok(
            ResponseLine {
                item_type,
                selector,
                server_name,
                server_port,
            }
        )
    }
}