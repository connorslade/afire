//! Some little functions used here and thare

use std::borrow::Cow;
use std::net::Ipv4Addr;

use crate::error::{Result, StartupError};

/// Decode a url encoded string.
/// Supports `+` and `%` encoding.
/// If the decode fails for any reason, [`None`] is returned.
pub fn decode_url(url: &str) -> Option<String> {
    let mut chars = url.chars();
    let mut out = String::with_capacity(url.len());

    while let Some(i) = chars.next() {
        match i {
            '+' => out.push(' '),
            '%' => {
                let mut hex = String::new();
                hex.push(chars.next()?);
                hex.push(chars.next()?);
                out.push(u8::from_str_radix(&hex, 16).ok()? as char);
            }
            _ => out.push(i),
        }
    }

    Some(out)
}

/// Encodes a string with url encoding.
/// Uses `%20` for spaces not `+`.
/// Allowed characters are `A-Z`, `a-z`, `0-9`, `-`, `.`, `_` and `~`.
pub fn encode_url(url: &str) -> String {
    const ALLOWED_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                   abcdefghijklmnopqrstuvwxyz\
                                   0123456789-._~";

    let mut out = String::with_capacity(url.len());

    for i in url.chars() {
        if i.is_ascii() && ALLOWED_CHARS.contains(&(i as u8)) {
            out.push(i);
            continue;
        }
        out.push_str(&format!("%{:02X}", i as u8));
    }

    out
}

/// Trait used to accept multiple types for the address of a server.
/// Default implementations are provided for `Ipv4Addr`, `String` and `&str`.
pub trait ToHostAddress {
    /// Convert the type to an `Ipv4Addr`.
    fn to_address(&self) -> Result<Ipv4Addr>;
}

impl ToHostAddress for Ipv4Addr {
    fn to_address(&self) -> Result<Ipv4Addr> {
        Ok(*self)
    }
}

impl ToHostAddress for String {
    fn to_address(&self) -> Result<Ipv4Addr> {
        Ok(Ipv4Addr::from(parse_ip(self)?))
    }
}

impl ToHostAddress for &str {
    fn to_address(&self) -> Result<Ipv4Addr> {
        Ok(Ipv4Addr::from(parse_ip(self)?))
    }
}

/// Parse a string to an IP address.
/// Will return a [`StartupError::InvalidIp`] if the IP has an invalid format.
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

/// Attempt to downcast a `Box<dyn Any>` to a `String` or `&str`.
/// Will return an empty string if the downcast fails.
pub(crate) fn any_string(any: Box<dyn std::any::Any + Send>) -> Cow<'static, str> {
    if let Some(i) = any.downcast_ref::<String>() {
        return Cow::Owned(i.to_owned());
    }

    if let Some(i) = any.downcast_ref::<&str>() {
        return Cow::Borrowed(i);
    }

    Cow::Borrowed("")
}

#[cfg(test)]
mod test {
    use crate::error::StartupError;

    use super::{decode_url, encode_url, parse_ip};

    #[test]
    fn test_url_decode() {
        assert_eq!(decode_url("hello+world").unwrap(), "hello world");
        assert_eq!(decode_url("hello%20world").unwrap(), "hello world");
        assert_eq!(
            decode_url("%3C%3E%22%23%25%7B%7D%7C%5C%5E~%5B%5D%60").unwrap(),
            "<>\"#%{}|\\^~[]`"
        );
    }

    #[test]
    fn test_url_decode_fail() {
        assert_eq!(decode_url("hello%20world%"), None);
        assert_eq!(decode_url("hello%20world%2"), None);
        assert_eq!(decode_url("hello%20world%2G"), None);
    }

    #[test]
    fn test_url_encode() {
        assert_eq!(encode_url("hello world"), "hello%20world");
        assert_eq!(encode_url("hello%20world"), "hello%2520world");
        assert_eq!(
            encode_url("<>\"#%{}|\\^~[]`"),
            "%3C%3E%22%23%25%7B%7D%7C%5C%5E~%5B%5D%60"
        );
    }

    #[test]
    fn test_parse_ip() {
        assert_eq!(parse_ip("123.231.43.3").unwrap(), [123, 231, 43, 3]);
        assert_eq!(parse_ip("123.231.43"), Err(StartupError::InvalidIp.into()));
        assert_eq!(
            parse_ip("256.231.43.3"),
            Err(StartupError::InvalidIp.into())
        );
    }
}
