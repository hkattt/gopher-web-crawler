use crate::COLON;

pub struct Request<'a> {
    pub selector: &'a str, 
    pub server_details: String,
}

impl<'a> Request<'a> {
    pub fn new(selector: &'a str, server_name: &'a str, server_port: u16) -> Request<'a> {
        let server_details = [server_name, COLON, &server_port.to_string()].concat();
        
        Request {
            selector,
            server_details,
        }
    }
}