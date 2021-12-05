#[cfg(any(feature = "rate_limit", feature = "logging"))]
use std::fmt;

/// Get Reason Phrase for a status code
///
/// Supports Status:
/// - 100-101
/// - 200-206
/// - 300-307
/// - 400-417
/// - 500-505
///
/// From https://www.w3.org/Protocols/rfc2616/rfc2616-sec6.html#sec6.1
pub(crate) fn reason_phrase(status: u16) -> String {
    match status {
        100 => "Continue",
        101 => "Switching Protocols",

        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        203 => "Non-Authoritative Information",
        204 => "No Content",
        205 => "Reset Content",
        206 => "Partial Content",

        300 => "Multiple Choices",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        305 => "Use Proxy",
        307 => "Temporary Redirect",

        400 => "Bad Request",
        401 => "Unauthorized",
        402 => "Payment Required",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
        407 => "Proxy Authentication Required",
        408 => "Request Time-out",
        409 => "Conflict",
        410 => "Gone",
        411 => "Length Required",
        412 => "Precondition Failed",
        413 => "Request Entity Too Large",
        414 => "Request-URI Too Large",
        415 => "Unsupported Media Type",
        416 => "Requested range not satisfiable",
        417 => "Expectation Failed",

        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Time-out",
        505 => "HTTP Version not supported",
        _ => "OK",
    }
    .to_string()
}

/// Remove the port from an address
///
/// '192.168.1.26:1234' -> '192.168.1.26'
#[cfg(any(feature = "rate_limit", feature = "logging"))]
pub(crate) fn remove_address_port<T>(address: T) -> String
where
    T: fmt::Display,
{
    address
        .to_string()
        .split(':')
        .next()
        .unwrap_or("null")
        .to_string()
}

/// Compares two Vectors
pub(crate) fn cmp_vec<T: std::cmp::PartialEq>(vec: &[T], vec2: &[T]) -> bool {
    if vec.len() != vec2.len() {
        return false;
    }

    for i in 0..vec.len() {
        if vec[i] != vec2[i] {
            return false;
        }
    }
    true
}

/// Decode a url encoded string
pub fn decode_url(url: String) -> String {
    // Convert input to Char array
    let url = url.chars().collect::<Vec<char>>();

    let mut res = String::new();
    let mut i = 0;
    while i < url.len() {
        if url[i] == '%' {
            let mut hex = String::new();
            try_push(&mut hex, url.get(i + 1));
            try_push(&mut hex, url.get(i + 2));
            res.push(u8::from_str_radix(&hex, 16).unwrap_or_default() as char);
            i += 3;
            continue;
        }
        try_push(&mut res, url.get(i));
        i += 1;
    }
    res
}

fn try_push(vec: &mut String, c: Option<&char>) {
    if let Some(c) = c {
        vec.push(*c);
    }
}
