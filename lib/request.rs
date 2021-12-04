use std::fmt;

use super::common::cmp_vec;
#[cfg(feature = "cookies")]
use super::cookie::Cookie;
use super::header::Header;
use super::method::Method;
use super::query::Query;

/// Http Request
#[derive(Hash, PartialEq, Eq)]
pub struct Request {
    /// Request method
    pub method: Method,

    /// Request path
    pub path: String,

    /// Request Query
    pub query: Query,

    /// Request headers
    pub headers: Vec<Header>,

    /// Request Cookies
    #[cfg(feature = "cookies")]
    pub cookies: Vec<Cookie>,

    /// Request body
    pub body: Vec<u8>,

    /// Client address
    pub address: String,

    /// Raw Http Request
    pub raw_data: Vec<u8>,
}

impl Request {
    /// Quick and easy way to create a request.
    ///
    /// ```rust
    /// use afire::{Request, Method, Query};
    ///
    /// let request = Request {
    ///    method: Method::GET,
    ///    path: "/".to_string(),
    ///    query: Query::new_empty(),
    ///    headers: vec![],
    ///    # #[cfg(feature = "cookies")]
    ///    cookies: vec![],
    ///    body: Vec::new(),
    ///    address: "127.0.0.1:8080".to_string(),
    ///    raw_data: Vec::new(),
    /// };
    ///
    /// # #[cfg(feature = "cookies")]
    /// assert!(request.compare(&Request::new(Method::GET, "/", Query::new_empty(), vec![], vec![], Vec::new(), "127.0.0.1:8080".to_string(), Vec::new())));
    /// # #[cfg(not(feature = "cookies"))]
    /// # assert!(request.compare(&Request::new(Method::GET, "/", Query::new_empty(), vec![], Vec::new(), "127.0.0.1:8080".to_string(), Vec::new())));
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        method: Method,
        path: &str,
        query: Query,
        headers: Vec<Header>,
        #[cfg(feature = "cookies")] cookies: Vec<Cookie>,
        body: Vec<u8>,
        address: String,
        raw_data: Vec<u8>,
    ) -> Request {
        Request {
            method,
            path: path.to_string(),
            query,
            headers,
            #[cfg(feature = "cookies")]
            cookies,
            body,
            address,
            raw_data,
        }
    }

    /// Get request body data as a string!
    pub fn body_string(&self) -> Option<String> {
        String::from_utf8(self.body.clone()).ok()
    }

    /// Get a request header by its name
    ///
    /// This is not case sensitive
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Request, Header, Method, Query};
    ///
    /// // Create Request
    /// # #[cfg(feature = "cookies")]
    /// let request = Request::new(Method::GET, "/", Query::new_empty(), vec![Header::new("hello", "world")], Vec::new(), Vec::new(), "127.0.0.1:8080".to_string(), Vec::new());
    /// # #[cfg(not(feature = "cookies"))]
    /// # let request = Request::new(Method::GET, "/", Query::new_empty(), vec![Header::new("hello", "world")], Vec::new(), "127.0.0.1:8080".to_string(), Vec::new());
    ///
    /// assert_eq!(request.header("hello").unwrap(), "world");
    /// ```
    pub fn header<T>(&self, name: T) -> Option<String>
    where
        T: fmt::Display,
    {
        let name = name.to_string().to_lowercase();
        for i in self.headers.clone() {
            if name == i.name.to_lowercase() {
                return Some(i.value);
            }
        }
        None
    }

    /// Compare two requests.
    pub fn compare(&self, other: &Request) -> bool {
        self.method == other.method
            && self.path == other.path
            && cmp_vec(&self.headers, &other.headers)
            && self.body == other.body
            && self.address == other.address
    }
}

// Impl Clone for Request
impl Clone for Request {
    fn clone(&self) -> Request {
        Request {
            method: self.method.clone(),
            path: self.path.clone(),
            query: self.query.clone(),
            headers: self.headers.clone(),
            #[cfg(feature = "cookies")]
            cookies: self.cookies.clone(),
            body: self.body.clone(),
            address: self.address.clone(),
            raw_data: self.raw_data.clone(),
        }
    }
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut headers = String::new();

        for header in &self.headers {
            headers.push_str(&format!("{}, ", header.to_string()));
        }

        let mut dbg = f.debug_struct("Request");
        dbg.field("method", &self.method);
        dbg.field("path", &self.path);
        dbg.field("query", &self.query);
        dbg.field("address", &self.address);
        dbg.field("headers", &headers);
        #[cfg(feature = "cookies")]
        dbg.field("cookies", &self.cookies);
        dbg.field("body", &self.body);
        dbg.finish()
    }
}
