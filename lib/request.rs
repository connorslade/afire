use std::fmt;

#[cfg(feature = "cookies")]
use crate::cookie::Cookie;
use crate::header::Header;
use crate::method::Method;
use crate::query::Query;

/// Http Request
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct Request {
    /// Request method
    pub method: Method,

    /// Request path
    pub path: String,

    /// Path Params
    #[cfg(feature = "path_patterns")]
    pub path_params: Vec<(String, String)>,

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
    /// Make a new Empty Request
    pub fn new_empty() -> Request {
        Request {
            method: Method::CUSTOM("NONE".to_owned()),
            path: "".to_owned(),
            path_params: Vec::new(),
            query: Query::new_empty(),
            headers: Vec::new(),
            #[cfg(feature = "cookies")]
            cookies: Vec::new(),
            body: Vec::new(),
            address: "".to_owned(),
            raw_data: Vec::new(),
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
    /// let request = Request {
    ///     method: Method::GET,
    ///     path: "/".to_owned(),
    ///     #[cfg(feature = "path_patterns")]
    ///     path_params: Vec::new(),
    ///     query: Query::new_empty(),
    ///     headers: vec![Header::new("hello", "world")],
    ///     #[cfg(feature = "cookies")]
    ///     cookies: Vec::new(),
    ///     body: Vec::new(),
    ///     address: "0.0.0.0".to_owned(),
    ///     raw_data: Vec::new(),
    /// };
    ///
    /// assert_eq!(request.header("hello").unwrap(), "world");
    /// ```
    pub fn header<T>(&self, name: T) -> Option<String>
    where
        T: AsRef<str>,
    {
        let name = name.as_ref().to_lowercase();
        for i in self.headers.clone() {
            if name == i.name.to_lowercase() {
                return Some(i.value);
            }
        }
        None
    }

    /// Get a path_params value
    ///
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Request, Response, Header, Method, Server};
    ///
    /// let mut server = Server::new("localhost", 8080);
    ///
    /// server.route(Method::GET, "/greet/{name}", |req| {
    ///     // Get name Path param
    ///     let name = req.path_param("name").unwrap();
    ///
    ///     // Make a nice Messgae
    ///     let message = format!("Hello, {}", name);
    ///
    ///     // Send Response
    ///     Response::new()
    ///         .text(message)
    ///         .header("Content-Type", "text/plain")
    /// });
    ///
    /// // Starts the server
    /// // This is blocking
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    #[cfg(feature = "path_patterns")]
    pub fn path_param<T>(&self, name: T) -> Option<String>
    where
        T: AsRef<str>,
    {
        let name = name.as_ref().to_owned();
        self.path_params
            .iter()
            .find(|x| x.0 == name)
            .map(|i| i.1.to_owned())
    }
}
