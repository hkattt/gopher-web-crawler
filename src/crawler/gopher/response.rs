use std::str;

use crate::{CRLF, DOT, TAB};

pub struct Response {
    pub buffer: Vec<u8>,    // Bytes received
    pub valid: bool,        // TODO: Explain
}

impl Response {
    pub fn new(buffer: Vec<u8>, valid: bool) -> Response {
        Response {
            buffer,
            valid,
        }
    }

    pub fn to_response_lines<'a>(&'a self) -> Vec<Option<ResponseLine<'a>>> {
        // Convert byte stream into a string (i.e. UTF-8 sequence)
        let buffer = match str::from_utf8(&self.buffer) {
            Ok(buffer) => buffer,
            Err(error) => panic!("Ivalid UTF-8 sequence: {error}"),
        };
        buffer.split(CRLF).map(|line| ResponseLine::new(line)).collect()
    }
}

pub enum ItemType {
    TXT,     // 0   Item is a text file
    DIR,     // 1   Item is a directory 
    ERR,     // 3   Item is a error
    BIN,     // 9   Item is a binary file
    DOT,     // .   Item is a . line
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
        // Line is a single dot
        if line.eq(DOT) {
            return Some(
                ResponseLine {
                    item_type: ItemType::DOT, 
                    selector: "", server_name: "", server_port: ""
                }
            );
        }

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