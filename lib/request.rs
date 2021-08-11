use std::fmt;

use super::common::cmp_vec;
use super::header::Header;
use super::method::Method;

/// Http Request
pub struct Request {
    pub method: Method,
    pub path: String,
    pub headers: Vec<Header>,
    pub body: String,
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
    /// };
    ///
    /// assert!(request.compare(&Request::new(Method::GET, "/", vec![], "".to_string())));
    /// ```
    pub fn new(method: Method, path: &str, headers: Vec<Header>, body: String) -> Request {
        Request {
            method,
            path: path.to_string(),
            headers,
            body,
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
            .field("headers", &headers)
            .field("body", &self.body)
            .finish()
    }
}
