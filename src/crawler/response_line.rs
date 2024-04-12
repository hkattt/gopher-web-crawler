use crate::{TAB, COLON};

pub enum ItemType {
    TXT,     // 0   Item is a text file
    DIR,     // 1   Item is a directory 
    ERR,     // 3   Item is a error
    BIN,     // 9   Item is a binary file
    UNKNOWN, // _    Item is unknown
}

pub struct ResponseLine<'a> {
    pub item_type:   ItemType,
    pub selector:    &'a str, 
    pub server_name: &'a str,
    pub server_port: &'a str,
}

impl<'a> ResponseLine<'a> {
    pub fn new(line: &'a str) -> Option<ResponseLine<'a>> {
        // TODO: What if there are not 4 tabs? Result NOne
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

    pub fn get_server_details(&self) -> String {
        [self.server_name, COLON, self.server_port].concat()
    }
}