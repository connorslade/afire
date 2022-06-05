//! Stuff for working with Raw HTTP data

use std::borrow::Cow;
use std::os::linux::raw;

#[cfg(feature = "path_decode_url")]
use crate::common;
#[cfg(feature = "cookies")]
use crate::cookie::Cookie;
use crate::header::Header;
use crate::method::Method;
use crate::query::Query;

//* NEW *//

/// (Method, Path, Query, Version)
pub fn parse_first_meta(str: &str) -> Option<(Method, String, Query, &str)> {
    let mut parts = str.split_whitespace();
    let raw_method = parts.next()?;
    let method = match raw_method.to_uppercase().as_str() {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "OPTIONS" => Method::OPTIONS,
        "HEAD" => Method::HEAD,
        "PATCH" => Method::PATCH,
        "TRACE" => Method::TRACE,
        _ => Method::CUSTOM(raw_method.to_owned()),
    };

    let mut raw_path = parts.next()?.chars();
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

    #[cfg(feature = "path_decode_url")]
    {
        final_path = common::decode_url(final_path)
    }

    Some((
        method,
        final_path,
        Query::from_body(final_query)?,
        parts.next()?,
    ))
}

//* NEW *//

/// Get the request method of a raw HTTP request.
///
/// Defaults to GET if no method found
pub fn get_request_method(raw_data: &str) -> Method {
    let method_str = match raw_data.split_once(' ') {
        Some(i) => i.0,
        None => return Method::GET,
    };

    match method_str.to_uppercase().as_str() {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "OPTIONS" => Method::OPTIONS,
        "HEAD" => Method::HEAD,
        "PATCH" => Method::PATCH,
        "TRACE" => Method::TRACE,
        _ => Method::CUSTOM(method_str.to_owned()),
    }
}

/// Get the path of a raw HTTP request.
pub fn get_request_path(raw_data: &str) -> String {
    let mut path_str = raw_data.splitn(3, ' ');

    let path = match path_str.nth(1) {
        Some(i) => i,
        None => return "/".to_owned(),
    };
    let path = match path.split_once('?') {
        Some(i) => i.0,
        None => path,
    };

    // Remove Consecutive slashes
    let mut new_path = String::new();
    for i in path.chars() {
        if i == '/' && new_path.chars().last().unwrap_or_default() != '/' {
            new_path.push('/');
            continue;
        }

        new_path.push(i);
    }

    // Trim trailing slash
    if new_path.chars().last().unwrap_or_default() == '/' {
        new_path.pop();
    }

    #[cfg(feature = "path_decode_url")]
    return common::decode_url(new_path);
    #[cfg(not(feature = "path_decode_url"))]
    return new_path;
}

/// Get The Query Data of a raw HTTP request.
pub fn get_request_query(raw_data: &str) -> Query {
    let mut path_str = raw_data.splitn(3, ' ');

    let path = match path_str.nth(1) {
        Some(i) => i,
        None => return Query::new_empty(),
    };
    let path = match path.split_once('?') {
        Some(i) => i.1,
        None => return Query::new_empty(),
    };

    Query::new(path).unwrap_or_else(|| Query(Vec::new()))
}

/// Get the body of a raw HTTP request.
pub fn get_request_body(raw_data: &[u8]) -> Vec<u8> {
    let mut raw_data = raw_data.iter().map(|x| x.to_owned());
    for _ in raw_data.clone() {
        // much jank
        if raw_data.next() == Some(b'\r')
            && raw_data.next() == Some(b'\n')
            && raw_data.next() == Some(b'\r')
            && raw_data.next() == Some(b'\n')
        {
            return raw_data.collect();
        }
    }

    Vec::new()
}

/// Get the headers of a raw HTTP request.
pub fn get_request_headers(raw_data: &str) -> Vec<Header> {
    let raw_headers = match raw_data.split_once("\r\n\r\n") {
        Some(i) => i.0,
        None => return Vec::new(),
    };

    let mut headers = Vec::new();
    for header in raw_headers.split("\r\n").skip(1) {
        if let Some(header) = Header::from_string(header.trim_matches(char::from(0))) {
            headers.push(header)
        }
    }

    headers
}

// TODO: Test This
/// Check if the socket connetion wants to use keep alive
pub fn connection_mode(headers: &[Header]) -> bool {
    matches!(headers.iter().find(|x| x.name == "Connection"), Some(i) if i.value == "keep-alive")
}

/// Get Cookies of a raw HTTP request.
#[cfg(feature = "cookies")]
pub fn get_request_cookies(raw_data: &str) -> Vec<Cookie> {
    let mut spilt = raw_data.split("\r\n\r\n");
    let raw_headers = spilt.next().unwrap_or_default().split("\r\n");

    for header in raw_headers {
        if !header.starts_with("Cookie:") {
            continue;
        }

        if let Some(cookie) = Cookie::from_string(header.trim_matches(char::from(0))) {
            return cookie;
        }
    }

    Vec::new()
}

/// Get the byte size of the headers of a raw HTTP request.
#[cfg(feature = "dynamic_resize")]
pub fn get_header_size(raw_data: &str) -> usize {
    match raw_data.split_once("\r\n\r\n") {
        Some(i) => i.0.len() + 4,
        None => raw_data.len(),
    }
}
