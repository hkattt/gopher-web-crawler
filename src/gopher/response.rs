use std::{
    str,
    rc::Rc
};

use crate::{CRLF, TAB};

/// Represents an item type offered by a Gopher server.
/// 
/// * `Txt`: 0  Item is a text file
/// * `Dir`: 1  Item is a directory
/// * `Err`: 3  Item is a error
/// * `Bin`: 9  Item is a binary file
/// * `Unknown`: Item type invalid or unsupported
pub enum ItemType {
    Txt,
    Dir,
    Err,
    Bin,
    Unknown,
}

/// Represents the outcome of a response from a Gopher server.
/// 
/// * `Complete`: The transaction completed sucessfully
/// * `Timeout`: The transaction failed because the read timed out
/// * `FileTooLong`: The transaction failed becasue the file was too long
/// * `ConnectionFailed`: The transaction failed because the connection failed
/// * `MissingEndLine`: The transaction failed because the response was missing 
/// the last line. This is only triggered for text and directory item types.
/// * `MalformedResponseLine`: The transaction failed because a response line was
/// malformed.
/// 
pub enum ResponseOutcome {
    Complete,
    Timeout,
    FileTooLong,
    ConnectionFailed,
    MissingEndLine,
    MalformedResponseLine,
}

/// Represents a response from a Gopher server. 
/// 
/// * `buffer`: Raw bytes received from the server, with the last line .\r\n removed
/// * `response_outcome`: Specifies the result of the transaction
pub struct Response {
    pub buffer: Vec<u8>,    // Bytes received
    pub response_outcome: ResponseOutcome,        // TODO: Explain
}

/// Represents a response line from a Gopher server.
/// 
/// * `item_type`: First character of the human-readable display string.
/// * `selector`: String being used to request the item
/// * `server_name`: Host name or IP address of the server providing the item
/// * `server_port`: The port number of the server providing the item
/// 
/// The display string of the response line is ignored.
pub struct ResponseLine{
    pub item_type:   ItemType,
    pub selector:    Rc<String>, 
    pub server_name: Rc<String>,
    pub server_port: u16,
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

impl Response {
    /// Constructs a new `Response` instance
    /// 
    /// # Arguments
    /// 
    /// * `buffer`: Bytes received from the server
    /// * `response_outcome`: Outcome of the transaction
    /// 
    /// # Returns
    /// 
    /// A new `Response`` instance
    pub fn new(buffer: Vec<u8>, response_outcome: ResponseOutcome) -> Response {
        Response {
            buffer,
            response_outcome,
        }
    }

    /// Splits the Gopher response into multiple response lines. Gopher response lines 
    /// are seperated by CLRF.
    pub fn to_response_lines<'a>(&'a self) -> Vec<Result<ResponseLine, ResponseLineError>> {
        // Convert byte stream into a string (i.e. UTF-8 sequence)
        let buffer = str::from_utf8(&self.buffer).expect("Ivalid UTF-8 sequence"); // TODO: Handle error??
        buffer.split(CRLF).map(|line| ResponseLine::new(line.to_string())).collect()
    }
}

impl<'a> ResponseLine {
    /// Constructs a new `ResponseLine` instance from a response line.
    /// 
    /// # Arguments
    /// 
    /// * `line`: Response line received from a Gopher server
    /// 
    /// # Returns
    /// 
    /// A `ResponseLine` if the response line is valid. Otherwise, returns the 
    /// appropriate `ResponseLineError`.
    pub fn new(line: String) -> Result<ResponseLine, ResponseLineError> {
        if line.is_empty() {
            return Err(ResponseLineError::Empty);
        }

        let mut parts = line.splitn(4, TAB).map(String::from).collect::<Vec<_>>();

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

/// Represents issues of a Gopher response line.
/// 
/// * `Empty`: The response line is a empty string
/// * `InvalidPart(line)`: The response line cannot be split into 4 parts
/// * `EmptyDisplayString(line)`: The display string is empty
/// * `EmptyHost(server_name, server_port, selector)`: The hostname is empty
/// * `NonIntPort(server_name, server_port, selector)`: The port number is
/// not an integer
#[derive(Debug)]
pub enum ResponseLineError {
    Empty,
    InvalidParts(String),
    EmptyDisplayString(String),
    EmptyHost(String, String, String), 
    NonIntPort(String, String, String)
}

impl<'a> std::error::Error for ResponseLineError {}

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