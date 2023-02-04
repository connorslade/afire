use std::{
    cell::RefCell,
    fmt::Debug,
    io::{BufRead, BufReader, Read},
    net::{SocketAddr, TcpStream},
};

use crate::{
    consts::BUFF_SIZE,
    error::{Result, StreamError},
    internal::http::parse_request_line,
    Cookie, Header, Method, Query,
};

/// Http Request
pub struct Request {
    /// Request method.
    pub method: Method,

    /// Request path (not tokenized).
    /// The query string is not included, its in the `query` field.
    pub path: String,

    /// HTTP version string.
    /// Should usally be "HTTP/1.1".
    pub version: String,

    /// Path Params, filled by the router
    pub path_params: RefCell<Vec<(String, String)>>,

    /// Request Query.
    pub query: Query,

    /// Request headers.
    /// Will not include cookies, which are in the `cookies` field.
    pub headers: Vec<Header>,

    /// Request Cookies.
    pub cookies: Vec<Cookie>,

    /// Request body, as a static byte vec.
    pub body: Vec<u8>,

    /// Client socket address.
    /// If you are using a reverse proxy, this will be the address of the proxy (often localhost).
    pub address: SocketAddr,
}

impl Request {
    /// Read a request from a TcpStream.
    pub(crate) fn from_socket(stream: &mut TcpStream) -> Result<Self> {
        trace!(Level::Debug, "Reading header");
        let peer_addr = stream.peer_addr()?;
        let mut reader = BufReader::new(stream);
        let mut request_line = Vec::with_capacity(BUFF_SIZE);
        reader
            .read_until(10, &mut request_line)
            .map_err(|_| StreamError::UnexpectedEof)?;

        let (method, path, query, version) = parse_request_line(&request_line)?;

        let mut headers = Vec::new();
        let mut cookies = Vec::new();
        loop {
            let mut buff = Vec::with_capacity(BUFF_SIZE);
            reader
                .read_until(10, &mut buff)
                .map_err(|_| StreamError::UnexpectedEof)?;
            let line = String::from_utf8_lossy(&buff);
            if line.len() <= 2 {
                break;
            }

            let header = Header::from_string(&line[..line.len() - 2])?;
            if header.name != "Cookie" {
                headers.push(header);
                continue;
            }

            if let Some(i) = Cookie::from_string(&header.value) {
                cookies.extend(i);
                continue;
            }

            headers.push(header);
        }

        let content_len = headers
            .iter()
            .find(|i| i.name.to_lowercase() == "content-length")
            .map(|i| i.value.parse::<usize>().unwrap_or(0))
            .unwrap_or(0);
        let mut body = vec![0; content_len];

        if content_len > 0 {
            reader
                .read_exact(&mut body)
                .map_err(|_| StreamError::UnexpectedEof)?;
        }

        Ok(Self {
            method,
            path,
            version,
            path_params: RefCell::new(Vec::new()),
            query,
            headers,
            cookies,
            body,
            address: peer_addr,
        })
    }

    pub(crate) fn keep_alive(&self) -> bool {
        self.header("Connection")
            .map(|i| i.to_lowercase() == "keep-alive")
            .unwrap_or(false)
    }

    /// Get a request header by its name.
    /// This is not case sensitive.
    /// ## Example
    /// ```rust
    /// # use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    /// # use std::sync::Arc;
    /// # use std::cell::RefCell;
    /// # use std::str::FromStr;
    /// # use afire::{Request, Header, Method, Query};
    /// // Create Request
    /// let request = Request {
    ///     /* SNIP */
    /// #   method: Method::GET,
    /// #   path: "/".to_owned(),
    /// #   version: "HTTP/1.1".to_owned(),
    /// #   path_params: RefCell::new(Vec::new()),
    /// #   query: Query::from_str("").unwrap(),
    ///     headers: vec![Header::new("Hello", "world")],
    /// #   cookies: Vec::new(),
    /// #   body: Vec::new(),
    /// #   address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5261),
    ///     /* SNIP */
    /// };
    ///
    /// assert_eq!(request.header("hello").unwrap(), "world");
    /// ```
    pub fn header<T>(&self, name: T) -> Option<&str>
    where
        T: AsRef<str>,
    {
        let name = name.as_ref().to_lowercase();
        for i in &self.headers {
            if name == i.name.to_lowercase() {
                return Some(&i.value);
            }
        }
        None
    }

    /// Get a path paramater by its name.
    ///
    /// ## Example
    /// ```rust
    /// # use afire::{Request, Response, Header, Method, Server, Content};
    /// # let mut server = Server::<()>::new("localhost", 8080);
    /// server.route(Method::GET, "/greet/{name}", |req| {
    ///     // Get name Path param
    ///     // This is safe to unwrap because the router will only call this handler if the path param exists
    ///     let name = req.param("name").unwrap();
    ///
    ///     // Format a nice Messgae
    ///     let message = format!("Hello, {}", name);
    ///
    ///     // Send Response
    ///     Response::new()
    ///         .text(message)
    ///         .content(Content::TXT)
    /// });
    /// ```
    pub fn param<T>(&self, name: T) -> Option<String>
    where
        T: AsRef<str>,
    {
        let name = name.as_ref().to_owned();
        self.path_params
            .borrow()
            .iter()
            .find(|x| x.0 == name)
            .map(|i| i.1.to_owned())
    }
}

impl Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Request")
            .field("method", &self.method)
            .field("path", &self.path)
            .field("version", &self.version)
            .field("path_params", &self.path_params.borrow())
            .field("query", &self.query)
            .field("headers", &self.headers)
            .field("cookies", &self.cookies)
            .field("body", &self.body)
            .field("address", &self.address)
            .finish()
    }
}
