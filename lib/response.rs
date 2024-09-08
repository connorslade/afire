use std::cell::RefCell;
use std::fmt::{self, Debug, Display, Formatter};
use std::io::{ErrorKind, Read, Write};
use std::mem;

use std::sync::Arc;

use crate::{
    consts,
    error::Result,
    header::headers_to_string,
    header::{HeaderName, Headers},
    internal::{
        handle::Writeable,
        socket::{Socket, SocketStream},
        sync::ForceLockMutex,
    },
    proto::http::status::Status,
    Content, Header, SetCookie,
};

/// Http Response
#[derive(Debug)]
pub struct Response {
    /// Response status code
    pub status: Status,

    /// Response Data.
    /// Can be either a Static `Vec<u8>` or a Stream (impl [`Read`])
    pub data: ResponseBody,

    /// List of response headers.
    /// This does not contain the default headers.
    pub headers: Headers,

    /// Response reason phrase.
    /// If this is None, the reason phrase will be automatically generated based on the status code.
    pub reason: Option<String>,

    /// Response Flags:
    /// - Close: Set the Connection header to close and will close the connection after the response is sent.
    /// - End: End the connection without sending a response
    pub flag: ResponseFlag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseFlag {
    /// No Flag
    None,
    /// Close the socket
    Close,
    /// End the connection without sending a response
    End,
}

/// Response Data.
/// Can be either a Static Vec<u8> or a Stream (impl [`Read`]).
/// Static responses are sent in one go, while streams are sent in chunks (chunked transfer encoding).
pub enum ResponseBody {
    Empty,
    Static(Vec<u8>),
    Stream(Writeable),
}

impl Response {
    /// Create a new Blank Response
    ///
    /// Default data is as follows
    /// - Status: 200
    /// - Data: OK
    /// - Headers: Vec::new()
    /// ## Example
    /// ```rust
    /// # use afire::{Response, Header};
    /// Response::new();
    /// ```
    pub fn new() -> Self {
        Self {
            status: Status::Ok,
            data: ResponseBody::Empty,
            headers: Default::default(),
            reason: None,
            flag: ResponseFlag::None,
        }
    }

    /// Creates a new Default Response with the End flag set.
    pub fn end() -> Self {
        Self {
            flag: ResponseFlag::End,
            ..Default::default()
        }
    }

    /// Add a status code to a Response.
    /// This accepts [`Status`] as well as a [`u16`].
    /// ## Example
    /// ```rust
    /// # use afire::{Response, Header, Status};
    /// // Create Response
    /// Response::new().status(Status::Ok);
    /// ```
    pub fn status(self, code: impl Into<Status>) -> Self {
        Self {
            status: code.into(),
            ..self
        }
    }

    /// Manually set the Reason Phrase.
    /// If this is not set, it will be inferred from the status code.
    /// Non standard status codes will have a reason phrase of "OK".
    /// ```rust
    /// # use afire::{Response, Header, Status};
    /// // Create Response
    /// let response = Response::new()
    ///     .status(Status::Ok)
    ///     .reason("Hello");
    /// ```
    pub fn reason(self, reason: impl AsRef<str>) -> Self {
        Self {
            reason: Some(reason.as_ref().to_owned()),
            ..self
        }
    }

    /// Add text as data to a Response.
    /// Will accept any type that implements Display, such as [`String`], [`str`], [`i32`], serde_json::Value, etc.
    /// This response type is considered static and will be sent in one go, not chunked.
    /// ## Example
    /// ```rust
    /// # use afire::Response;
    /// // Create Response
    /// let response = Response::new()
    ///    .text("Hello from afire!");
    /// ```
    pub fn text(self, text: impl Display) -> Self {
        Self {
            data: text.to_string().as_bytes().to_vec().into(),
            ..self
        }
    }

    /// Add raw bytes as data to a Response.
    /// This response type is considered static and will be sent in one go, not chunked.
    /// ## Example
    /// ```rust
    /// # use afire::Response;
    /// // Create Response
    /// let response = Response::new()
    ///   .bytes([79, 75]); // Bytes for "OK"
    /// ```
    pub fn bytes(self, bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            data: bytes.into().into(),
            ..self
        }
    }

    /// Add a stream as data to a Response.
    /// This response type is considered dynamic and will be streamed to the client in chunks using `Transfer-Encoding: chunked`.
    /// ## Example
    /// ```rust,no_run
    /// # use afire::{Response, Method, Server, error::Result};
    /// # use std::fs::File;
    /// # fn run() -> Result<()> {
    /// const PATH: &str = "path/to/file.txt";
    /// let mut server = Server::builder("localhost", 8080, ()).build()?;
    ///
    /// server.route(Method::GET, "/download-stream", |ctx| {
    ///     let stream = File::open(PATH)?;
    ///     ctx.stream(stream).send()?;
    ///     Ok(())
    /// });
    /// # Ok(())
    /// # }
    /// ```
    pub fn stream(self, stream: impl Read + Send + 'static) -> Self {
        Self {
            data: ResponseBody::Stream(Box::new(RefCell::new(stream))),
            ..self
        }
    }

    /// Add a Header to a Response.
    /// Will accept any type that implements `Into<Header>`, so you can use a tuple of `(impl Into<HeaderName>, impl AsRef<str>)` like `(&str, &str)` or a [header struct][`crate::header`] (recommended).
    /// ## Example
    /// ```
    /// # use afire::prelude::*;
    /// # use afire::headers::Server;
    /// // Create Response
    /// let response = Response::new()
    ///     // Set 'X-Test' header to 'Test'
    ///    .header(("X-Test", "Test"))
    ///     // Set 'Server' header to 'teapot'
    ///    .header(Server::new("teapot"));
    /// #
    /// ```
    pub fn header(mut self, header: impl Into<Header>) -> Self {
        self.headers.push(header.into());
        self
    }

    /// Add a list of Headers to a Response.
    /// Only accepts a slice of [`Header`]s.
    /// ## Example
    /// ```rust
    /// # use afire::{Response, Header};
    /// // Create Response
    /// let response = Response::new()
    ///     .headers([
    ///         Header::new("Content-Type", "text/html"),
    ///         Header::new("Test-Header", "Test-Value")
    ///     ].to_vec());
    /// ```
    pub fn headers(mut self, headers: Vec<Header>) -> Self {
        self.headers.extend(headers);
        self
    }

    /// Will set the `Connection: close` header on the Response.
    /// Then it will close the connection after the Response has been sent.
    /// ## Example
    /// ```rust
    /// # use afire::{Response};
    /// // Create Response
    /// let response = Response::new()
    ///   .text("goodbye!")
    ///   .close();
    /// ```
    pub fn close(self) -> Self {
        Self {
            flag: ResponseFlag::Close,
            ..self
        }
    }

    /// Add a cookie to a response.
    /// The [`SetCookie`] will be converted to a [`Header`] and added to the Response.
    /// ## Example
    /// ```
    /// # use afire::{Response, SetCookie};
    /// // Create Response and add cookie
    /// let response = Response::new()
    ///     .cookie(SetCookie::new("name", "value"))
    ///     .cookie(SetCookie::new("name2", "value2"));
    /// ```
    pub fn cookie(mut self, cookie: SetCookie) -> Self {
        self.headers
            .push(Header::new("Set-Cookie", cookie.to_string()));
        self
    }

    /// Add a list of cookies to a response.
    /// ## Example
    /// ```
    /// # use afire::{Response, SetCookie};
    /// // Create Response and add cookie
    /// let response = Response::new()
    ///     .cookies(&[
    ///         SetCookie::new("name", "value"),
    ///         SetCookie::new("name2", "value2")
    ///     ]);
    /// ```
    pub fn cookies(self, cookie: &[SetCookie]) -> Self {
        let mut new = Vec::new();

        for c in cookie {
            new.push(Header::new("Set-Cookie", c.to_string()));
        }

        self.headers(new)
    }

    /// Set a Content Type on a Response with a [`Content`] enum.
    /// This will add a `Content-Type` header to the Response.
    /// ## Example
    /// ```
    /// # use afire::{Response, Content};
    /// // Create Response and type
    /// let response = Response::new()
    ///     .content(Content::HTML);
    /// ```
    pub fn content(mut self, content_type: Content) -> Self {
        self.headers.push(content_type.into());
        self
    }

    /// Lets you modify the Response with a function before it is sent to the client.
    /// This can be used to have middleware that modifies the Response on specific routes.
    pub fn modifier(mut self, modifier: impl FnOnce(&mut Response)) -> Self {
        modifier(&mut self);
        self
    }

    /// Writes a Response to a TcpStream.
    /// Will take care of adding default headers and closing the connection if needed.
    pub fn write(
        &mut self,
        raw_stream: Arc<Socket>,
        default_headers: &[Header],
        content_length: bool,
    ) -> Result<()> {
        // Add default headers to response
        // Only the ones that aren't already in the response
        for i in default_headers {
            if !self.headers.has(&i.name) {
                self.headers.push(i.clone());
            }
        }

        let static_body = self.data.is_static();

        // Add content-length header to response if we are sending a static body
        if content_length && static_body && !self.headers.has(HeaderName::ContentLength) {
            self.headers.push(self.data.content_len());
        }

        // Add Connection: close if response is set to close
        if self.flag == ResponseFlag::Close && !self.headers.has(HeaderName::Connection) {
            self.headers
                .push(Header::new(HeaderName::Connection, "close"));
        }

        if !static_body && !self.headers.has(HeaderName::TransferEncoding) {
            self.headers
                .push(Header::new(HeaderName::TransferEncoding, "chunked"));
        }

        // Convert the response to a string
        let response = format!(
            "HTTP/1.1 {} {}\r\n{}\r\n\r\n",
            self.status.code(),
            self.reason
                .as_deref()
                .unwrap_or_else(|| self.status.reason_phrase()),
            headers_to_string(&self.headers)
        );

        let mut stream = raw_stream.force_lock();
        let error = match stream.write_all(response.as_bytes()) {
            Ok(_) => self.data.write(&mut stream),
            Err(e) => Err(e.into()),
        };
        drop(stream);
        raw_stream.unlock();

        error
    }
}

impl Default for Response {
    fn default() -> Response {
        Response::new()
    }
}

impl ResponseBody {
    pub(crate) fn take(&mut self) -> ResponseBody {
        mem::replace(self, ResponseBody::Empty)
    }

    /// Checks if the ResponseBody is static.
    fn is_static(&self) -> bool {
        matches!(self, ResponseBody::Static(_) | ResponseBody::Empty)
    }

    /// Gets the content length header of a static ResponseBody.
    /// If the ResponseBody is not static it will panic.
    fn content_len(&self) -> Header {
        let len = match self {
            ResponseBody::Static(data) => data.len(),
            ResponseBody::Empty => 0,
            _ => unreachable!("Can't get content length of a stream"),
        };
        Header::new("Content-Length", len.to_string())
    }

    /// Writes a ResponseBody to a TcpStream.
    /// Either in one go if it is static or in chunks if it is a stream.
    fn write(&mut self, stream: &mut SocketStream) -> Result<()> {
        match self {
            ResponseBody::Empty => {}
            ResponseBody::Static(data) => stream.write_all(data)?,
            ResponseBody::Stream(data) => {
                let data = data.get_mut();
                loop {
                    let mut chunk = vec![0; consts::CHUNK_SIZE];
                    let read = match data.read(&mut chunk) {
                        Ok(0) => break,
                        Ok(n) => n,
                        Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                        Err(e) => return Err(e.into()),
                    };

                    let mut section = format!("{read:X}\r\n").as_bytes().to_vec();
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

impl Debug for ResponseBody {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.debug_tuple("Empty").finish(),
            Self::Static(arg) => f.debug_tuple("Static").field(arg).finish(),
            Self::Stream(_arg) => f.debug_tuple("Stream").finish(),
        }
    }
}
