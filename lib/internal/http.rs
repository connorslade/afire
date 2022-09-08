//! Stuff for working with Raw HTTP data

#[cfg(feature = "cookies")]
use crate::cookie::Cookie;
use crate::header::Header;
use crate::method::Method;
use crate::query::Query;

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
