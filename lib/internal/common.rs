//! Some little functions used here and thare

use crate::{
    error::{Result, StartupError},
    Header,
};

/// Get Reason Phrase for a status code
///
/// Supports Status:
/// - 100-101
/// - 200-206
/// - 300-307
/// - 400-417
/// - 500-505
///
/// From <https://www.w3.org/Protocols/rfc2616/rfc2616-sec6.html#sec6.1>\
pub fn reason_phrase(status: u16) -> String {
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

/// Parse a string to an IP address
pub fn parse_ip(raw: &str) -> Result<[u8; 4]> {
    if raw == "localhost" {
        return Ok([127, 0, 0, 1]);
    }

    let mut ip = [0; 4];
    let split_ip = raw.split('.').collect::<Vec<&str>>();

    if split_ip.len() != 4 {
        return Err(StartupError::InvalidIp.into());
    }

    for i in 0..4 {
        let octet = split_ip[i]
            .parse::<u8>()
            .map_err(|_| StartupError::InvalidIp)?;
        ip[i] = octet;
    }

    Ok(ip)
}

#[inline]
fn try_push(vec: &mut String, c: Option<&char>) {
    if let Some(c) = c {
        vec.push(*c);
    }
}

#[inline]
pub(crate) fn has_header(headers: &[Header], name: &str) -> bool {
    headers.iter().any(|x| x.name == name)
}

pub(crate) fn any_string(any: Box<dyn std::any::Any + Send>) -> String {
    if let Some(i) = any.downcast_ref::<String>() {
        return i.to_owned();
    }

    if let Some(i) = any.downcast_ref::<&str>() {
        return i.to_owned().to_owned();
    }

    "".to_owned()
}

#[doc(hidden)]
pub fn trace(str: String) {
    #[cfg(feature = "tracing")]
    println!("{}", str);
}

/// Internal Debug Printing
///
/// Enabled with the `tracing` feature
#[macro_export]
macro_rules! trace {
    ($($arg : tt) +) => {
        $crate::internal::common::trace(format!($($arg)+));
    };
}
