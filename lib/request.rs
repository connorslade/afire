use std::{
    cell::RefCell,
    fmt::Debug,
    io::{BufRead, BufReader, Read},
    net::{SocketAddr, TcpStream},
    str::FromStr,
};

use crate::{
    consts::BUFF_SIZE,
    error::{ParseError, Result, StreamError},
    header::{HeaderType, Headers},
    Cookie, Error, Header, Method, Query,
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
    pub headers: Headers,

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
            if header.name != HeaderType::Cookie {
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
            .find(|i| i.name == HeaderType::ContentLength)
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
            headers: Headers(headers),
            cookies,
            body,
            address: peer_addr,
        })
    }

    pub(crate) fn keep_alive(&self) -> bool {
        self.headers
            .get(HeaderType::Connection)
            .map(|i| i.to_lowercase() == "keep-alive")
            .unwrap_or(false)
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
    pub fn param(&self, name: impl AsRef<str>) -> Option<String> {
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

/// Parse a request line into a method, path, query, and version
pub(crate) fn parse_request_line(bytes: &[u8]) -> Result<(Method, String, Query, String)> {
    let request_line = String::from_utf8_lossy(bytes);
    let mut parts = request_line.split_whitespace();

    let raw_method = match parts.next() {
        Some(i) => i,
        None => return Err(Error::Parse(ParseError::NoMethod)),
    };
    let method =
        Method::from_str(raw_method).map_err(|_| Error::Parse(ParseError::InvalidMethod))?;
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

    let query = match Query::from_str(&final_query) {
        Ok(i) => i,
        Err(_) => return Err(Error::Parse(ParseError::InvalidQuery)),
    };

    let version = match parts.next() {
        Some(i) => i.to_owned(),
        None => return Err(Error::Parse(ParseError::NoVersion)),
    };

    Ok((method, final_path, query, version))
}
