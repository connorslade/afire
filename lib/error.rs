use std::{fmt, io};

use crate::{Method, Request};

pub enum Error {
    Handle(HandleError),
    Parse(ParseError),
    Io(io::Error),
}

/// Errors thet can arize while handling a request
#[derive(Debug, Clone)]
pub enum HandleError {
    /// Error readint the stream
    StreamRead,

    /// Route matching request path not found
    NotFound(Method, String),

    /// A route or middleware paniced while running
    Panic(Request, String),
}

#[derive(Debug, Clone)]
pub enum ParseError {
    StreamRead,
    NoSeparator,
    NoMethod,
    NoPath,
    NoVersion,
    NoRequestLine,
    InvalidQuery,
    InvalidHeader(usize),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut err = f.debug_struct("Error");

        match self {
            Error::Handle(e) => err.field("Handle", e),
            Error::Parse(e) => err.field("Parse", e),
            Error::Io(e) => err.field("Io", &e.kind()),
        };

        err.finish()
    }
}
