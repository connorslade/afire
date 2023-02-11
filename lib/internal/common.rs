//! Some little functions used here and there

use std::borrow::Cow;
use std::net::Ipv4Addr;

use crate::error::{Result, StartupError};

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

impl ToHostAddress for [u8; 4] {
    fn to_address(&self) -> Result<Ipv4Addr> {
        Ok(Ipv4Addr::new(self[0], self[1], self[2], self[3]))
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

    use super::parse_ip;

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
