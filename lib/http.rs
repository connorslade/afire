//! Stuff for wioking with Raw HTTP data

#[cfg(feature = "cookies")]
use super::cookie::Cookie;
use crate::header::Header;
use crate::method::Method;
use crate::query::Query;

/// Get the request method of a raw HTTP request.
pub fn get_request_method(raw_data: String) -> Method {
    let method_str = raw_data
        .split(' ')
        .next()
        .unwrap_or("GET")
        .to_string()
        .to_uppercase();

    match method_str.as_str() {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "OPTIONS" => Method::OPTIONS,
        "HEAD" => Method::HEAD,
        "PATCH" => Method::PATCH,
        "TRACE" => Method::TRACE,
        _ => Method::CUSTOM(method_str),
    }
}

/// Get the path of a raw HTTP request.
pub fn get_request_path(raw_data: String) -> String {
    let mut path_str = raw_data.split(' ');

    let path = path_str.nth(1).unwrap_or_default().to_string();
    let mut path = path.split('?');
    let mut new_path = String::new();

    // Remove Consecutive slashes
    for i in path.next().unwrap_or_default().chars() {
        if i != '/' {
            new_path.push(i);
            continue;
        }

        if new_path.chars().last().unwrap_or_default() != '/' {
            new_path.push('/');
        }
    }

    // Trim trailing slash
    if new_path.chars().last().unwrap_or_default() == '/' {
        new_path.pop();
    }
    new_path
}

// Get The Query Data of a raw HTTP request.
pub fn get_request_query(raw_data: String) -> Query {
    let mut path_str = raw_data.split(' ');
    if path_str.clone().count() <= 1 {
        return Query::new_empty();
    }

    let path = path_str.nth(1).unwrap_or_default().to_string();
    let mut path = path.split('?');

    if path.clone().count() <= 1 {
        return Query::new_empty();
    }
    Query::new(path.nth(1).unwrap_or_default())
}

/// Get the body of a raw HTTP request.
pub fn get_request_body(raw_data: String) -> String {
    let mut data = raw_data.split("\r\n\r\n");

    if data.clone().count() >= 2 {
        return data
            .nth(1)
            .unwrap_or_default()
            .trim_matches(char::from(0))
            .to_string();
    }
    "".to_string()
}

/// Get the headers of a raw HTTP request.
pub fn get_request_headers(raw_data: String) -> Vec<Header> {
    let mut headers = Vec::new();
    let mut spilt = raw_data.split("\r\n\r\n");
    let raw_headers = spilt.next().unwrap_or_default().split("\r\n");

    for header in raw_headers {
        if let Some(header) = Header::from_string(header.trim_matches(char::from(0))) {
            headers.push(header)
        }
    }

    headers
}

/// Get Cookies of a raw HTTP request.
#[cfg(feature = "cookies")]
pub fn get_request_cookies(raw_data: String) -> Vec<Cookie> {
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
pub fn get_header_size(raw_data: String) -> usize {
    let mut headers = raw_data.split("\r\n\r\n");
    headers.next().unwrap_or_default().len() + 4
}
