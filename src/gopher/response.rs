use std::{
    str,
    rc::Rc
};

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

    pub fn to_response_lines<'a>(&'a self) -> Vec<Result<ResponseLine, ResponseLineError>> {
        // Convert byte stream into a string (i.e. UTF-8 sequence)
        let buffer = str::from_utf8(&self.buffer).expect("Ivalid UTF-8 sequence"); // TODO: Handle error??
        buffer.split(CRLF).map(|line| ResponseLine::new(line.to_string())).collect()
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
    // TODO: Would these be better as strings?
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

impl<'a> std::error::Error for ResponseLineError {}

pub struct ResponseLine{
    pub item_type:   ItemType,
    pub selector:    Rc<String>, 
    pub server_name: Rc<String>,
    pub server_port: u16,
}

impl<'a> ResponseLine {
    pub fn new(line: String) -> Result<ResponseLine, ResponseLineError> {
        if line.is_empty() {
            return Err(ResponseLineError::Empty);
        }

        let mut parts = line.splitn(4, TAB).map(String::from).collect::<Vec<_>>();

        // TODO: Can we do this without cloning?
        if parts.len() != 4 {
            return Err(ResponseLineError::InvalidParts(line));
        }

        let user_display_string = parts.remove(0);
        let selector = parts.remove(0);
        let server_name = parts.remove(0);
        let server_port_str = parts.remove(0);

        let item_type = match user_display_string.chars().next() {
            Some(i) => match i {
                '0' => ItemType::Txt,
                '1' => ItemType::Dir,
                '3' => ItemType::Err,
                '9' => ItemType::Bin,
                _   => ItemType::Unknown
            },
            None => return Err(ResponseLineError::EmptyDisplayString(line))
        };
        // Server name cannot be empty        
        if server_name.is_empty() {
            return Err(ResponseLineError::EmptyHost(server_name, server_port_str, selector))
        }
        // Server port must be an integer        
        let server_port = server_port_str.parse::<u16>();
        let server_port = match server_port {
            Ok(port) => port,
            Err(_) => return Err(ResponseLineError::NonIntPort(server_name, server_port_str, selector)), 
        };

        Ok(
            ResponseLine {
                item_type,
                selector: Rc::new(selector),
                server_name: Rc::new(server_name),
                server_port,
            }
        )
    }
}