use std::fmt;

use super::common::cmp_vec;
use super::header::Header;
use super::method::Method;

/// Http Request
pub struct Request {
    /// Request method
    pub method: Method,

    /// Request path
    pub path: String,

    /// Request headers
    pub headers: Vec<Header>,

    /// Request body
    pub body: String,

    /// Client address
    pub address: String,

    /// Raw Http Request
    pub raw_data: String,
}

impl Request {
    /// Quick and easy way to create a request.
    ///
    /// ```rust
    /// use afire::{Request, Method};
    ///
    /// let request = Request {
    ///    method: Method::GET,
    ///    path: "/".to_string(),
    ///    headers: vec![],
    ///    body: "".to_string(),
    ///    address: "127.0.0.1:8080".to_string(),
    ///    raw_data: "".to_string(),
    /// };
    ///
    /// assert!(request.compare(&Request::new(Method::GET, "/", vec![], "".to_string(), "127.0.0.1:8080".to_string(), "".to_string())));
    /// ```
    pub fn new(
        method: Method,
        path: &str,
        headers: Vec<Header>,
        body: String,
        address: String,
        raw_data: String,
    ) -> Request {
        Request {
            method,
            path: path.to_string(),
            headers,
            body,
            address,
            raw_data,
        }
    }

    /// Compare two requests.
    ///
    /// ```rust
    /// use afire::{Request, Method};

    pub fn compare(&self, other: &Request) -> bool {
        self.method == other.method
            && self.path == other.path
            && cmp_vec(&self.headers, &other.headers)
            && self.body == other.body
            && self.address == other.address
    }
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut headers = String::new();

        for header in &self.headers {
            headers.push_str(&format!("{}, ", header.to_string()));
        }

        f.debug_struct("Request")
            .field("method", &self.method)
            .field("path", &self.path)
            .field("address", &self.address)
            .field("headers", &headers)
            .field("body", &self.body)
            .finish()
    }
}
