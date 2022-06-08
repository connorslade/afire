#[cfg(feature = "cookies")]
use crate::cookie::Cookie;
use crate::{
    common,
    error::{Error, ParseError},
    Header, Method, Query,
};

/// Http Request
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct Request {
    /// Request method
    pub method: Method,

    /// Request path
    pub path: String,

    /// HTTP version
    pub version: String,

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
    /// Parse an HTTP request into a [`Request]
    pub fn from_bytes(bytes: &[u8], address: String) -> Result<Self, Error> {
        // Find the \r\n\r\n to only parse the request 'metadata' (path, headers, etc)
        let meta_end_index = match (0..bytes.len() - 3).find(|i| {
            bytes[*i] == 0x0D
                && bytes[i + 1] == 0x0A
                && bytes[i + 2] == 0x0D
                && bytes[i + 3] == 0x0A
        }) {
            Some(i) => i,
            None => return Err(Error::Parse(ParseError::NoSeparator)),
        };
        // Turn the meta bytes into a string
        let meta_string = String::from_utf8_lossy(&bytes[0..meta_end_index]);
        let mut lines = meta_string.lines();

        // Parse the first like to get the method, path, query and verion
        let first_meta = match lines.next() {
            Some(i) => i,
            None => return Err(Error::Parse(ParseError::NoRequestLine)),
        };
        let (method, path, query, version) = parse_first_meta(first_meta)?;

        // Parse headers
        let mut headers = Vec::new();
        let mut cookies = Vec::new();
        for (i, e) in lines.enumerate() {
            headers.push(match Header::from_string(e) {
                Some(j) => {
                    if j.name == "Cookie" {
                        cookies.extend(parse_cookie(j.clone()))
                    }
                    j
                }
                None => return Err(Error::Parse(ParseError::InvalidHeader(i))),
            });
        }

        Ok(Request {
            method,
            path,
            version: version.to_owned(),
            path_params: Vec::new(),
            query,
            headers,
            cookies,
            body: bytes[meta_end_index..].to_vec(),
            address,
            raw_data: bytes.to_vec(),
        })
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
    ///     version: "HTTP/1.1".to_owned(),
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
    /// let mut server = Server::<()>::new("localhost", 8080);
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

/// (Method, Path, Query, Version)
fn parse_first_meta(str: &str) -> Result<(Method, String, Query, &str), Error> {
    let mut parts = str.split_whitespace();
    let raw_method = match parts.next() {
        Some(i) => i,
        None => return Err(Error::Parse(ParseError::NoMethod)),
    };
    let method = match raw_method.to_uppercase().as_str() {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "OPTIONS" => Method::OPTIONS,
        "HEAD" => Method::HEAD,
        "PATCH" => Method::PATCH,
        "TRACE" => Method::TRACE,
        _ => Method::CUSTOM(raw_method.to_owned()),
    };

    let mut raw_path = match parts.next() {
        Some(i) => i.chars(),
        None => return Err(Error::Parse(ParseError::NoVersion)),
    };
    let mut final_path = String::new();
    let mut final_query = String::new();
    let mut last_is_slash = false;
    while let Some(i) = raw_path.next() {
        match i {
            '/' | '\\' => {
                if last_is_slash {
                    continue;
                }

                last_is_slash = true;
                final_path.push('/');
            }
            '?' => {
                final_query.extend(raw_path);
                break;
            }
            _ => {
                last_is_slash = false;
                final_path.push(i);
            }
        }
    }

    #[cfg(feature = "path_decode_url")]
    {
        final_path = common::decode_url(final_path)
    }

    let query = match Query::from_body(final_query) {
        Some(i) => i,
        None => return Err(Error::Parse(ParseError::InvalidQuery)),
    };

    let version = match parts.next() {
        Some(i) => i,
        None => return Err(Error::Parse(ParseError::NoVersion)),
    };

    Ok((method, final_path, query, version))
}

fn parse_cookie(header: Header) -> Vec<Cookie> {
    let mut final_cookies = Vec::new();
    for i in header.value.split(';') {
        let mut cookie_parts = i.splitn(2, '=');
        let name = match cookie_parts.next() {
            Some(i) => i.trim(),
            None => continue,
        };

        let value = match &cookie_parts.next() {
            Some(i) => i.trim(),
            None => continue,
        };

        final_cookies.push(Cookie::new(
            common::decode_url(name.to_owned()),
            common::decode_url(value.to_owned()),
        ));
    }

    final_cookies
}
