use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::io::{Read, Write};
use std::net::TcpStream;

use crate::{
    common::{has_header, reason_phrase},
    error::Result,
    header::headers_to_string,
    internal::handle::Writeable,
    Content, Header, SetCookie,
};

/// Http Response
pub struct Response {
    /// Response status code
    pub status: u16,

    /// Response Data as Bytes
    data: ResponseBody,

    /// Response Headers
    pub headers: Vec<Header>,

    /// Response Reason
    pub reason: Option<String>,

    /// Force Close Connection
    pub close: bool,
}

enum ResponseBody {
    Static(Vec<u8>),
    Stream(Writeable),
}

impl Response {
    /// Create a new Blank Response
    ///
    /// Default data is as follows
    /// - Status: 200
    ///
    /// - Data: OK
    ///
    /// - Headers: Vec::new()
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Response, Header};
    /// // Create Response
    /// let response = Response::new();
    /// ```
    pub fn new() -> Self {
        Self {
            status: 200,
            data: vec![79, 75].into(),
            headers: Vec::new(),
            reason: None,
            close: false,
        }
    }

    /// Add a status to a Response
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Response, Header};
    ///
    /// // Create Response
    /// let response = Response::new()
    ///    .status(200); // <- Here it is
    /// ```
    pub fn status(self, code: u16) -> Self {
        Self {
            status: code,
            ..self
        }
    }

    /// Manually set the Reason Phrase
    /// ```rust
    /// // Import Library
    /// use afire::{Response, Header};
    ///
    /// // Create Response
    /// let response = Response::new()
    ///    .reason("OK");
    /// ```
    pub fn reason<T>(self, reason: T) -> Self
    where
        T: AsRef<str>,
    {
        Self {
            reason: Some(reason.as_ref().to_owned()),
            ..self
        }
    }

    /// Add text as data to a Response
    ///
    /// Will accept any type that implements Display
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::Response;
    ///
    /// // Create Response
    /// let response = Response::new()
    ///    .text("Hi :P");
    /// ```
    pub fn text<T>(self, text: T) -> Self
    where
        T: Display,
    {
        Self {
            data: text.to_string().as_bytes().to_vec().into(),
            ..self
        }
    }

    /// Add raw bytes as data to a Response
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::Response;
    ///
    /// // Create Response
    /// let response = Response::new()
    ///   .bytes(&[79, 75]);
    /// ```
    pub fn bytes(self, bytes: &[u8]) -> Self {
        Self {
            data: bytes.to_vec().into(),
            ..self
        }
    }

    /// Add a stream as data to a Response
    /// It will be streamed to the client in chunks using `Transfer-Encoding: chunked`.
    /// ## Example
    /// ```rust,no_run
    /// # const PATH: &str = "path/to/file.txt";
    /// // Import Library
    /// use afire::{Response, Method, Server};
    /// use std::fs::File;
    /// 
    /// let mut server = Server::<()>::new("localhost", 8080);
    /// 
    /// server.route(Method::GET, "/download-stream", |_| {
    ///     let stream = File::open(PATH).unwrap();
    ///     Response::new().stream(stream)
    /// });
    /// 
    /// server.start().unwrap();
    /// ```
    pub fn stream<T>(self, stream: T) -> Self
    where
        T: Read + Send + 'static,
    {
        Self {
            data: ResponseBody::Stream(Box::new(RefCell::new(stream))),
            ..self
        }
    }

    /// Add a Header to a Response
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Response, Header};
    ///
    /// // Create Response
    /// let response = Response::new()
    ///    .header("Content-Type", "text/html");
    /// ```
    pub fn header<T, K>(self, key: T, value: K) -> Self
    where
        T: AsRef<str>,
        K: AsRef<str>,
    {
        let mut new_headers = self.headers;
        new_headers.push(Header::new(key, value));

        Self {
            headers: new_headers,
            ..self
        }
    }

    /// Add a Vec of Headers to a Response
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Response, Header};
    ///
    /// // Create Response
    /// let response = Response::new()
    ///   .headers(&[Header::new("Content-Type", "text/html")]);
    /// ```
    pub fn headers(self, headers: &[Header]) -> Self {
        let mut new_headers = self.headers;
        new_headers.append(&mut headers.to_vec());

        Self {
            headers: new_headers,
            ..self
        }
    }

    /// Close the connection without sendng a Response
    ///
    /// Will ignore any other options defined on the Response
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Response};
    ///
    /// // Create Response
    /// let response = Response::new()
    ///   .close();
    /// ```
    pub fn close(self) -> Self {
        Self {
            close: true,
            ..self
        }
    }

    /// Add a cookie to a response.
    /// ## Example
    /// ```
    /// // Import Library
    /// use afire::{Response, SetCookie};
    ///
    /// // Create Response and add cookie
    /// let response = Response::new()
    ///     .cookie(SetCookie::new("name", "value"));
    /// ```
    pub fn cookie(self, cookie: SetCookie) -> Self {
        let mut new = self;
        new.headers
            .push(Header::new("Set-Cookie", cookie.to_string()));
        new
    }

    /// Add a vec of cookies to a response.
    /// ## Example
    /// ```
    /// // Import Library
    /// use afire::{Response, SetCookie};
    ///
    /// // Create Response and add cookie
    /// let response = Response::new()
    ///     .cookies(&[SetCookie::new("name", "value")]);
    /// ```
    pub fn cookies(self, cookie: &[SetCookie]) -> Self {
        let mut new = Vec::new();

        for c in cookie {
            new.push(Header::new("Set-Cookie", c.to_string()));
        }

        self.headers(&new)
    }

    /// Set a Content Type on a Response
    /// ## Example
    /// ```
    /// // Import Library
    /// use afire::{Response, Content};
    ///
    /// // Create Response and type
    /// let response = Response::new()
    ///     .content(Content::HTML);
    /// ```
    pub fn content(mut self, content_type: Content) -> Self {
        self.headers
            .push(Header::new("Content-Type", content_type.as_type()));
        self
    }

    pub(crate) fn write(
        mut self,
        stream: &mut TcpStream,
        default_headers: &[Header],
    ) -> Result<()> {
        // Add default headers to response
        // Only the ones that arent already in the response
        for i in default_headers {
            if !has_header(&self.headers, &i.name) {
                self.headers.push(i.clone());
            }
        }

        let static_body = self.data.is_static();

        // Add content-length header to response if we are sending a static body
        if static_body && !has_header(&self.headers, "Content-Length") {
            self.headers.push(self.data.content_len());
        }

        // Add Connection: close if response is set to close
        if self.close && !has_header(&self.headers, "Connection") {
            self.headers.push(Header::new("Connection", "close"));
        }

        if !static_body && !has_header(&self.headers, "Transfer-Encoding") {
            self.headers
                .push(Header::new("Transfer-Encoding", "chunked"));
        }

        // Convert the response to a string
        let response = format!(
            "HTTP/1.1 {} {}\r\n{}\r\n\r\n",
            self.status,
            self.reason
                .to_owned()
                .unwrap_or_else(|| reason_phrase(self.status)),
            headers_to_string(self.headers)
        )
        .as_bytes()
        .to_vec();

        stream.write_all(&response)?;

        self.data.write(stream)?;

        Ok(())
    }
}

impl Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Response")
            .field("status", &self.status)
            .field("headers", &self.headers)
            .field("reason", &self.reason)
            .field("close", &self.close)
            .finish()
    }
}

impl Default for Response {
    fn default() -> Response {
        Response::new()
    }
}

impl ResponseBody {
    fn is_static(&self) -> bool {
        matches!(self, ResponseBody::Static(_))
    }

    fn content_len(&self) -> Header {
        let len = match self {
            ResponseBody::Static(data) => data.len(),
            _ => unreachable!("Can't get content length of a stream"),
        };
        Header::new("Content-Length", len.to_string())
    }

    fn write(self, stream: &mut TcpStream) -> Result<()> {
        match self {
            ResponseBody::Static(data) => stream.write_all(&data)?,
            ResponseBody::Stream(mut data) => {
                let data = data.get_mut();
                loop {
                    let mut chunk = vec![0; 16 * 1024];
                    let read = data.read(&mut chunk)?;
                    if read == 0 {
                        break;
                    }

                    let mut section = format!("{:X}\r\n", read).as_bytes().to_vec();
                    section.extend(&chunk[..read]);
                    section.extend(b"\r\n");

                    stream.write_all(&section)?;
                }

                stream.write_all(b"0\r\n\r\n")?;
            }
        };

        Ok(())
    }
}

impl From<Vec<u8>> for ResponseBody {
    fn from(x: Vec<u8>) -> Self {
        ResponseBody::Static(x)
    }
}

impl From<Writeable> for ResponseBody {
    fn from(x: Writeable) -> Self {
        ResponseBody::Stream(x)
    }
}
