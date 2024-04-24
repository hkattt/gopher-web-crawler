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
        let buffer = str::from_utf8(&self.buffer).expect("Ivalid UTF-8 sequence"); // TODO: Handle error??
        buffer.split(CRLF).map(|line| ResponseLine::new(line)).collect()
    }

    // TODO: Finish this func. We need to check for \r\n dot \r\n
    // Call this thing somewhere useful hopefully
    pub fn clean_buffer(&mut self) {
        let mut found = false;
        let mut last_byte = b'\0';

        println!("CLEANING BUFFER!!");

        for (i, &byte) in self.buffer.iter().enumerate().rev() {
            // Found a dot followed by a carriage return
            if byte == b'.' && last_byte == b'\r' {
                found = true;
            }
            else if byte == b'\n' {
                // \n dot \r found
                if found {
                    self.buffer.truncate(i);
                    return;
                } 
            } else {
                found = false;
            }
            last_byte = byte;
        }
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