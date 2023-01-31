//! Stuff for working with Raw HTTP data

use std::str::FromStr;

use crate::{
    error::{ParseError, Result},
    Error, Method, Query,
};

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
