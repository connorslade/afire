//! Errors that can occur in the process of connecting to clients, parsing HTTP and handling requests.

use std::{
    error,
    fmt::{self, Display, Formatter},
    io, result,
    sync::Arc,
};

use crate::{HeaderName, Method};

/// Easy way to use a Result<T, [`crate::Error`]>
pub type Result<T> = result::Result<T, Error>;
pub(crate) type AnyResult<T = ()> = result::Result<T, Box<dyn error::Error>>;

/// A generic error type for afire.
/// Contains variants for [Startup][`StartupError`], [Stream][`StreamError`], [Handle][`HandleError`], [Parse][`ParseError`], [IO][`std::io::Error`] and [Miscellaneous][`String`] errors.
#[derive(Debug, Clone)]
pub enum Error {
    /// Error while starting the server.
    Startup(StartupError),

    /// Stream error.
    Stream(StreamError),

    /// Error while handling a Request.
    Handle(HandleError),

    /// Error while parsing request HTTP.
    Parse(ParseError),

    /// IO Errors.
    Io(Arc<io::Error>),

    /// Miscellaneous errors.
    Misc(String),

    /// Response does not exist (probably because of an error with the request).
    None,
}

/// Errors that can occur while starting the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartupError {
    /// The IP address specified is invalid.
    /// This can happen if the IP address string a server is supposed to listen on is invalid, or not an IPV4 address.
    InvalidIp,

    /// The socket timeout specified is invalid (must be greater than 0).
    InvalidSocketTimeout,

    /// A [forbidden header][`crate::header::FORBIDDEN_HEADERS`] was set as a default header.
    ForbiddenDefaultHeader {
        /// The header in question.
        header: HeaderName,
    },

    /// Errors that can occur while parsing a route path.
    Path {
        ///The error that occurred.
        error: PathError,
        /// The route path that caused the error.
        route: String,
    },
}

/// Errors that can occur while parsing a route path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
    /// Parameter, Wildcard and AnyAfter segments cannot be adjacent in route paths as they make matching ambiguous.
    /// For example, `/hello{a}{b}` is ambiguous because it could match `/helloworld!` as:
    /// - { a: 'world!', b: '' }
    /// - { a: 'world', b: '!' }
    /// - { a: 'worl', b: 'd!' }
    /// - { a: 'wor', b: 'ld!' }
    /// - etc.
    AmbiguousPath,

    /// Parameter segments must be terminated with a closing curly-bracket.
    UnterminatedParameter,

    /// Parameters cannot be nested inside each other or contain '{' or '}'.
    NestedParameter,
}

/// Errors that can arise while handling a request.
#[derive(Debug, Clone)]
pub enum HandleError {
    /// Route matching request was found, but did not send a response.
    /// This happens if a route handler returns Ok(()) before sending a response.
    /// If you want to send a response asynchronously after the route handler returns, you can use [`crate::Context::guarantee_will_send`] to promise the router that you will *eventually* send a response.
    NotImplemented,

    /// Route tried to send a response, but one was already sent.
    /// For more information, see [`crate::Context::send`].
    ResponseAlreadySent,

    /// Route matching request path not found.
    NotFound(Method, String),
}

/// Error that can occur while parsing the HTTP of a request
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// No `\r\n\r\n` found in request to separate metadata from body.
    NoSeparator,

    /// No Method found in request HTTP.
    NoMethod,

    /// No Path found in request HTTP.
    NoPath,

    /// No Version found in request HTTP.
    NoVersion,

    /// No Request Line found in HTTP.
    NoRequestLine,

    /// Invalid Method in Request HTTP.
    /// The only supported methods are GET, POST, PUT, DELETE, OPTIONS, HEAD, PATCH, and TRACE.
    InvalidMethod,
    /// Invalid Header in Request HTTP.
    InvalidHeader,

    /// Invalid HTTP Version in Request HTTP.
    /// The only supported versions are 1.0 and 1.1.
    InvalidHttpVersion,

    /// The HOST header was not found in the request.
    /// It is required by HTTP/1.1.
    NoHostHeader,
}

/// Error that can occur while reading or writing to a stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamError {
    /// The stream ended unexpectedly.
    UnexpectedEof,
}

impl Error {
    /// Create a new [`Error::Misc`] with the given message.
    /// This is a shorthand for `Err(Error::Misc(msg))`.
    /// # Examples
    /// ```
    /// # use afire::prelude::*;
    /// # fn test(server: &mut Server) {
    /// server.route(Method::GET, "/", |ctx| {
    ///     if ctx.req.body.len() > 100 {
    ///         return Error::bail("Request body too big!")?;
    ///     }
    ///
    ///     ctx.text("Hello World!").send()?;
    ///     Ok(())
    /// });
    /// # }
    /// ```
    pub fn bail<T>(msg: impl Into<String>) -> Result<T> {
        Err(Error::Misc(msg.into()))
    }
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::Handle(e) => fmt::Display::fmt(e, f),
            Error::Startup(e) => fmt::Display::fmt(e, f),
            Error::Stream(e) => fmt::Display::fmt(e, f),
            Error::Parse(e) => fmt::Display::fmt(e, f),
            Error::Io(e) => fmt::Display::fmt(e, f),
            Error::Misc(e) => fmt::Display::fmt(e, f),
            Error::None => f.write_str("None"),
        }
    }
}

impl Display for HandleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            HandleError::NotImplemented => f.write_str("Route handler did not send a response"),
            HandleError::ResponseAlreadySent => {
                f.write_str("Route handler tried to send a response, but one was already sent")
            }
            HandleError::NotFound(method, path) => {
                f.write_fmt(format_args!("No route found at {method} {path}"))
            }
        }
    }
}

impl Display for StartupError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            StartupError::Path { error, route } => {
                f.write_fmt(format_args!("Path error {error} on route `{route}`."))
            }
            StartupError::InvalidIp => f.write_str("The IP address specified is invalid"),
            StartupError::InvalidSocketTimeout => {
                f.write_str("The socket timeout specified is invalid (must be greater than 0)")
            }
            StartupError::ForbiddenDefaultHeader { header } => f.write_fmt(format_args!(
                "The header `{header}` is forbidden and may not be used as a default header",
            )),
        }
    }
}

impl Display for PathError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            PathError::AmbiguousPath => f.write_str(
                "Any, AnyAfter, and Parameter segments cannot be adjacent as they make matching ambiguous"
            ),
            PathError::UnterminatedParameter => f.write_str(
                "Parameter segments must be terminated with a closing curly-bracket",
            ),
            PathError::NestedParameter => f.write_str(
                "Parameters cannot be nested inside each other or contain '{' or '}'"
            ),
      }
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
        Error::Handle(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(Arc::new(e))
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Error::Startup(l0), Error::Startup(r0)) => l0 == r0,
            (Error::Stream(l0), Error::Stream(r0)) => l0 == r0,
            (Error::Handle(l0), Error::Handle(r0)) => l0 == r0,
            (Error::Parse(l0), Error::Parse(r0)) => l0 == r0,
            (Error::Io(l0), Error::Io(r0)) => l0.kind() == r0.kind(),
            (Error::Misc(l0), Error::Misc(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Eq for HandleError {}

impl PartialEq for HandleError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (HandleError::NotFound(m1, p1), HandleError::NotFound(m2, p2)) => m1 == m2 && p1 == p2,
            _ => false,
        }
    }
}
