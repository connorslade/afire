//! Errors that can occur in the process of connectioning to clients, parseing HTTP and handling requests.

use std::{io, result};

use crate::{Method, Request};

/// Easy way to use a Result<T, Error>
pub type Result<T> = result::Result<T, Error>;

/// Errors that can occur,,,
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Error while handling a Request
    Handle(Box<HandleError>),

    /// Error while parsing request HTTP
    Parse(ParseError),

    /// IO Errors
    Io(io::ErrorKind),

    /// Response does not exist (probably because of an error with the request)
    None,
}

/// Errors thet can arize while handling a request
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandleError {
    /// Route matching request path not found
    NotFound(Method, String),

    /// A route or middleware paniced while running
    Panic(Box<Result<Request>>, String),
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

    /// Invalid Header in Request HTTP
    InvalidHeader(usize),
}
