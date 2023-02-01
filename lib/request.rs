use std::{
    io::{BufRead, BufReader, Read},
    net::{SocketAddr, TcpStream},
    sync::Arc,
};

use crate::{
    consts::BUFF_SIZE,
    error::{Result, StreamError},
    internal::http::parse_request_line,
    Cookie, Header, Method, Query,
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
    pub path_params: Vec<(String, String)>,

    /// Request Query
    pub query: Query,

    /// Request headers
    pub headers: Vec<Header>,

    /// Request Cookies
    pub cookies: Vec<Cookie>,

    /// Request body
    pub body: Arc<Vec<u8>>,

    /// Client address
    pub address: SocketAddr,
}

impl Request {
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
            path_params: Vec::new(),
            query,
            headers,
            cookies,
            body: Arc::new(body),
            address: peer_addr,
        })
    }

    pub(crate) fn keep_alive(&self) -> bool {
        self.header("Connection")
            .map(|i| i.to_lowercase() == "keep-alive")
            .unwrap_or(false)
    }

    /// Get a request header by its name
    ///
    /// This is not case sensitive
    /// ## Example
    /// ```rust
    /// use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    /// use std::sync::Arc;
    ///
    /// // Import Library
    /// use afire::{Request, Header, Method, Query};
    ///
    /// // Create Request
    /// let request = Request {
    ///     method: Method::GET,
    ///     path: "/".to_owned(),
    ///     version: "HTTP/1.1".to_owned(),
    ///     path_params: Vec::new(),
    ///     query: Query::new_empty(),
    ///     headers: vec![Header::new("hello", "world")],
    ///     cookies: Vec::new(),
    ///     body: Arc::new(Vec::new()),
    ///     address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5261),
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

    /// Get a path_params value
    ///
    /// ## Example
    /// ```rust,no_run
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
    /// server.start().unwrap();
    /// ```
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
