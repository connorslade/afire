use std::{
    borrow::Cow,
    fmt::{self, Debug, Display},
    io::{BufRead, BufReader, Read},
    net::SocketAddr,
    str::FromStr,
    sync::Arc,
};

use crate::{
    consts::BUFF_SIZE,
    cookie::CookieJar,
    error::{ParseError, Result, StreamError},
    header::{HeaderName, Headers},
    internal::{socket::Socket, sync::ForceLockMutex},
    Cookie, Error, Header, Method, Query,
};

/// Http Request
pub struct Request {
    /// Request method.
    pub method: Method,

    /// Request path (not tokenized).
    /// The query string is not included, its in the `query` field.
    pub path: String,

    /// HTTP version of the current connection.
    /// Will typically be HTTP/1.1.
    pub version: HttpVersion,

    /// Request Query.
    pub query: Query,

    /// Request headers.
    /// Will not include cookies, which are in the `cookies` field.
    pub headers: Headers,

    /// Request Cookies.
    pub cookies: CookieJar,

    /// Request body, as a static byte vec.
    pub body: Arc<Vec<u8>>,

    /// Client socket address.
    /// If you are using a reverse proxy, this will be the address of the proxy (often localhost).
    pub address: SocketAddr,

    /// The raw tcp socket
    pub socket: Arc<Socket>,
}

/// HTTP Version of the current connection.
/// Will typically be HTTP/1.1 but can also be HTTP/1.0.
/// Other versions are not supported, and will abort the connection.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum HttpVersion {
    Http10,
    Http11,
}

impl Request {
    pub(crate) fn keep_alive(&self) -> bool {
        let connection = self.headers.get(HeaderName::Connection);
        match self.version {
            // Only keep-alive if the connection header specifies
            HttpVersion::Http10 => connection
                .map(|i| i.to_lowercase() == "keep-alive")
                .unwrap_or(false),
            // Keep-alive unless the connection header specifies close
            HttpVersion::Http11 => connection
                .map(|i| i.to_lowercase() != "close")
                .unwrap_or(true),
        }
    }

    /// Gets the body of the request as a string.
    /// This uses the [`String::from_utf8_lossy`] method, so it will replace invalid UTF-8 characters with the unicode replacement character (�).
    /// If you want to use a different encoding or handle invalid characters, use a string method on the body field.
    pub fn body_str(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.body)
    }

    /// Read a request from a TcpStream.
    pub(crate) fn from_socket(raw_stream: Arc<Socket>) -> Result<Self> {
        let mut stream = raw_stream.force_lock();

        trace!(Level::Debug, "Reading header");
        let peer_addr = stream.peer_addr()?;
        let mut reader = BufReader::new(&mut *stream);
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
            if header.name != HeaderName::Cookie {
                headers.push(header);
                continue;
            }

            cookies.extend(Cookie::from_string(&header.value));
        }

        let content_len = headers
            .iter()
            .find(|i| i.name == HeaderName::ContentLength)
            .map(|i| i.value.parse::<usize>().unwrap_or(0))
            .unwrap_or(0);
        let mut body = vec![0; content_len];

        if content_len > 0 {
            reader
                .read_exact(&mut body)
                .map_err(|_| StreamError::UnexpectedEof)?;
        }

        drop(stream);

        let headers = Headers(headers);
        if version >= HttpVersion::Http11 && !headers.has(HeaderName::Host) {
            return Err(Error::Parse(ParseError::NoHostHeader));
        }

        Ok(Self {
            method,
            path,
            version,
            query,
            headers,
            cookies: CookieJar(cookies),
            body: Arc::new(body),
            address: peer_addr,
            socket: raw_stream,
        })
    }
}

/// Parse a request line into a method, path, query, and version
pub(crate) fn parse_request_line(bytes: &[u8]) -> Result<(Method, String, Query, HttpVersion)> {
    let request_line = String::from_utf8_lossy(bytes);
    let mut parts = request_line.split_whitespace();

    let raw_method = parts.next().ok_or(Error::Parse(ParseError::NoMethod))?;
    let method =
        Method::from_str(raw_method).map_err(|_| Error::Parse(ParseError::InvalidMethod))?;
    let mut raw_path = parts
        .next()
        .ok_or(Error::Parse(ParseError::NoPath))?
        .chars();

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

    let query = Query::from_str(&final_query);
    let version = parts
        .next()
        .ok_or(Error::Parse(ParseError::NoVersion))?
        .parse()?;

    Ok((method, final_path, query, version))
}

impl Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Request")
            .field("method", &self.method)
            .field("path", &self.path)
            .field("version", &self.version)
            .field("query", &self.query)
            .field("headers", &self.headers)
            .field("cookies", &*self.cookies)
            .field("body", &self.body)
            .field("address", &self.address)
            .finish()
    }
}

impl FromStr for HttpVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "HTTP/1.0" => Ok(Self::Http10),
            "HTTP/1.1" => Ok(Self::Http11),
            _ => Err(Error::Parse(ParseError::InvalidHttpVersion)),
        }
    }
}

impl Display for HttpVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http10 => write!(f, "HTTP/1.0"),
            Self::Http11 => write!(f, "HTTP/1.1"),
        }
    }
}

impl Debug for HttpVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http10 => write!(f, "HTTP/1.0"),
            Self::Http11 => write!(f, "HTTP/1.1"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::request::HttpVersion;

    #[test]
    fn test_http_ordering() {
        assert_eq!(HttpVersion::Http10, HttpVersion::Http10);
        assert_eq!(HttpVersion::Http11, HttpVersion::Http11);

        assert!(HttpVersion::Http10 < HttpVersion::Http11);
        assert!(HttpVersion::Http11 > HttpVersion::Http10);
        assert!(HttpVersion::Http10 <= HttpVersion::Http11);
        assert!(HttpVersion::Http11 >= HttpVersion::Http10);
    }
}
