//! Errors that can occur in the process of connecting to clients, parsing HTTP and handling requests.

use std::{
    error,
    fmt::{self, Display, Formatter},
    rc::Rc,
    result,
};

use crate::{Method, Request, Response};

/// Easy way to use a Result<T, [`crate::Error`]>
pub type Result<T> = result::Result<T, Error>;
pub(crate) type AnyResult<T = ()> = result::Result<T, Box<dyn error::Error>>;

/// Errors that can occur at startup or in the process of connecting to clients, parsing HTTP and handling requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Error while starting the server
    Startup(StartupError),

    /// Stream error
    Stream(StreamError),

    /// Error while handling a Request
    Handle(Box<HandleError>),

    /// Error while parsing request HTTP
    Parse(ParseError),

    /// IO Errors
    Io(String),

    /// Response does not exist (probably because of an error with the request)
    None,
}

/// Errors that can occur while starting the server
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartupError {
    /// The IP address specified is invalid
    InvalidIp,

    /// The socket timeout specified is invalid (must be greater than 0)
    InvalidSocketTimeout,
}

/// Errors that can arise while handling a request
#[derive(Debug, Clone)]
pub enum HandleError {
    /// Route matching request path not found
    NotFound(Method, String),

    /// A route or middleware panicked while running
    Panic(Box<Result<Rc<Request>>>, String),
}

/// Error that can occur while parsing the HTTP of a request
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// No `\r\n\r\n` found in request to separate metadata from body
    NoSeparator,

    /// No Method found in request HTTP
    NoMethod,

    /// No Path found in request HTTP
    NoPath,

    /// No Version found in request HTTP
    NoVersion,

    /// No Request Line found in HTTP
    NoRequestLine,

    /// Invalid Query in Path
    InvalidQuery,

    /// Invalid Method in Request HTTP
    InvalidMethod,

    /// Invalid Header in Request HTTP
    InvalidHeader,

    /// Invalid HTTP Version in Request HTTP.
    /// The only supported versions are 1.0 and 1.1.
    InvalidHttpVersion,

    /// The HOST header was not found in the request.
    /// It is required by HTTP/1.1.
    NoHostHeader,
}

/// Error that can occur while reading or writing to a stream
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamError {
    /// The stream ended unexpectedly
    UnexpectedEof,
}

impl error::Error for Error {}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::Handle(e) => fmt::Display::fmt(e, f),
            Error::Startup(e) => fmt::Display::fmt(e, f),
            Error::Stream(e) => fmt::Display::fmt(e, f),
            Error::Parse(e) => fmt::Display::fmt(e, f),
            Error::Io(e) => f.write_str(e),
            Error::None => f.write_str("None"),
        }
    }
}

impl Display for HandleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            HandleError::NotFound(method, path) => {
                f.write_fmt(format_args!("No route found at {method} {path}"))
            }
            HandleError::Panic(_req, err) => {
                f.write_fmt(format_args!("Route handler panicked: {err}"))
            }
        }
    }
}

impl Display for StartupError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            StartupError::InvalidIp => "The IP address specified is invalid",
            StartupError::InvalidSocketTimeout => {
                "The socket timeout specified is invalid (must be greater than 0)"
            }
        })
    }
}

impl Display for StreamError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            StreamError::UnexpectedEof => "The stream ended unexpectedly",
        })
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ParseError::NoSeparator => {
                r"No `\r\n\r\n` found in request to separate metadata from body"
            }
            ParseError::NoMethod => "No Method found in request HTTP",
            ParseError::NoPath => "No Path found in request HTTP",
            ParseError::NoVersion => "No Version found in request HTTP",
            ParseError::NoRequestLine => "No Request Line found in HTTP",
            ParseError::InvalidQuery => "Invalid Query in Path",
            ParseError::InvalidMethod => "Invalid Method in Request HTTP",
            ParseError::InvalidHeader => "Invalid Header in Request HTTP",
            ParseError::InvalidHttpVersion => "Request HTTP Version is not supported",
            ParseError::NoHostHeader => "The Host header was not found in the request",
        })
    }
}

impl From<StartupError> for Error {
    fn from(e: StartupError) -> Self {
        Error::Startup(e)
    }
}

impl From<StreamError> for Error {
    fn from(e: StreamError) -> Self {
        Error::Stream(e)
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Error::Parse(e)
    }
}

impl From<HandleError> for Error {
    fn from(e: HandleError) -> Self {
        Error::Handle(Box::new(e))
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.to_string())
    }
}

impl Eq for HandleError {}
impl PartialEq for HandleError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (HandleError::NotFound(m1, p1), HandleError::NotFound(m2, p2)) => m1 == m2 && p1 == p2,
            (HandleError::Panic(_, s1), HandleError::Panic(_, s2)) => s1 == s2,
            _ => false,
        }
    }
}
