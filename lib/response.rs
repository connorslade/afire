use super::header::Header;

/// Http Response
pub struct Response {
    pub status: u16,
    pub data: String,
    pub headers: Vec<Header>,
}

impl Response {
    /// Quick and easy way to create a response.
    pub fn new(status: u16, data: &str, headers: Vec<Header>) -> Response {
        Response {
            status,
            data: data.to_string(),
            headers: headers,
        }
    }
}
