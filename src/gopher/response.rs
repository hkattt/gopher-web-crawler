use std::str;

use crate::{CRLF, TAB};

pub enum ResponseOutcome {
    Complete,
    Timeout,
    FileTooLong,
    ConnectionFailed,
    MissingEndLine,
}

impl ToString for ResponseOutcome {
    fn to_string(&self) -> String {
        match self {
            ResponseOutcome::Complete => String::from("Completed sucessfully"),
            ResponseOutcome::Timeout  => String::from("Connection timed out"),
            ResponseOutcome::FileTooLong => String::from("File too long"),
            ResponseOutcome::ConnectionFailed => String::from("Failed to connect"),
            ResponseOutcome::MissingEndLine => String::from("Missing end-line"),
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

    pub fn to_response_lines<'a>(&'a self) -> Vec<Option<ResponseLine<'a>>> {
        // Convert byte stream into a string (i.e. UTF-8 sequence)
        let buffer = str::from_utf8(&self.buffer).expect("Ivalid UTF-8 sequence"); // TODO: Handle error??
        buffer.split(CRLF).map(|line| ResponseLine::new(line)).collect()
    }
}

// TODO: Change to lower case?
pub enum ItemType {
    TXT,     // 0   Item is a text file
    DIR,     // 1   Item is a directory 
    ERR,     // 3   Item is a error
    BIN,     // 9   Item is a binary file
    UNKNOWN, // _   Item is unknown     
}

pub struct ResponseLine<'a> {
    pub item_type:   ItemType,
    pub selector:    &'a str, 
    pub server_name: &'a str,
    pub server_port: &'a str,
}

impl<'a> ResponseLine<'a> {
    pub fn new(line: &'a str) -> Option<ResponseLine<'a>> {
        let mut parts = line.splitn(4, TAB);

        // TODO: Can we do this without cloning?
        if parts.clone().count() != 4 {return None;}

        let user_display_string = parts.next().unwrap();
        
        let mut item_type = ItemType::UNKNOWN;
        match user_display_string.chars().next() {
            Some(i) => match i {
                '0' => item_type = ItemType::TXT,
                '1' => item_type = ItemType::DIR,
                '3' => item_type = ItemType::ERR,
                '9' => item_type = ItemType::BIN,
                _   => ()
            },
            None => return None
        };
        Some(
            ResponseLine {
                item_type,
                selector:    parts.next().unwrap(),
                server_name: parts.next().unwrap(),
                server_port: parts.next().unwrap(),
            }
        )
    }
}