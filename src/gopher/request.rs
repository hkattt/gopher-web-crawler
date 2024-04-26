use super::response::ItemType;

use::std::rc::Rc;

/// Represents a request to be sent to a Gopher server
/// 
/// * `selector`: String being used to request the item
/// * `server_details`: hostname:port of the server
/// * `item_type`: The type of item being requested
/// 
pub struct Request {
    pub selector: Rc<String>, 
    pub server_details: Rc<String>,
    pub item_type: ItemType
}

impl Request {
    /// Construct a new `Request` instance.
    /// 
    /// # Arguments
    /// * `selector`: String being used to request the item
    /// * `server_name`: The name of the server that has the item
    /// * `server_port`: The port number of the server providing the item
    /// 
    /// # Returns
    /// A new `Request` instance with `server_details`: `server_name`:`server_port`
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