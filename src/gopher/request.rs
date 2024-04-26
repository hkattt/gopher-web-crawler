use super::response::ItemType;

use::std::rc::Rc;

pub struct Request {
    pub selector: Rc<String>, 
    pub server_details: Rc<String>,
    pub item_type: ItemType
}

impl Request {
    pub fn new(selector: Rc<String>, server_name: Rc<String>, server_port: u16, item_type: ItemType) -> Request {
        let server_details = Rc::new(
            format!("{}:{}", server_name, server_port.to_string())
        );
        
        Request {
            selector,
            server_details,
            item_type,
        }
    }
}