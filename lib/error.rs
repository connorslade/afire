use std::{fmt, io};

use crate::{Method, Request, Response};

#[derive(Debug, Clone)]
pub enum Error {
    Handle(HandleError),
    Parse(ParseError),
    Io(io::ErrorKind),
}

/// Errors thet can arize while handling a request
#[derive(Debug, Clone)]
pub enum HandleError {
    /// Route matching request path not found
    NotFound(Method, String),

    /// A route or middleware paniced while running
    Panic(Request, String),
}

#[derive(Debug, Clone)]
pub enum ParseError {
    NoSeparator,
    NoMethod,
    NoPath,
    NoVersion,
    NoRequestLine,
    InvalidQuery,
    InvalidHeader(usize),
}