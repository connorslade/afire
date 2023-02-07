//! Some little functions used here and thare

use std::borrow::Cow;
use std::net::Ipv4Addr;

use crate::error::{Result, StartupError};

/// Decode a url encoded string.
/// Supports `+` and `%` encoding
pub fn decode_url(url: &str) -> String {
    #[inline]
    fn try_push(vec: &mut String, c: Option<&char>) {
        if let Some(c) = c {
            vec.push(*c);
        }
    }

    // Convert input to Char array
    let url = url.chars().collect::<Vec<char>>();

    let mut res = String::new();
    let mut i = 0;
    while i < url.len() {
        if url[i] == '+' {
            res.push(' ');
            i += 1;
            continue;
        }
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

    use super::{decode_url, parse_ip};

    #[test]
    fn test_url_decode() {
        assert_eq!(decode_url("hello+world"), "hello world");
        assert_eq!(decode_url("hello%20world"), "hello world");
        assert_eq!(
            decode_url("%3C%3E%22%23%25%7B%7D%7C%5C%5E%7E%5B%5D%60"),
            "<>\"#%{}|\\^~[]`"
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
