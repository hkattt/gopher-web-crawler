use super::response::ItemType;

pub struct Request<'a> {
    pub selector: &'a str, 
    pub server_details: String,
    pub item_type: ItemType
}

impl<'a> Request<'a> {
    pub fn new(selector: &'a str, server_name: &'a str, server_port: u16, item_type: ItemType) -> Request<'a> {
        let server_details = format!("{}:{}", server_name, server_port.to_string());
        
        Request {
            selector,
            server_details,
            item_type,
        }
    }
}