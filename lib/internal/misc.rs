//! Some little functions used here and there

use std::fmt::{self, Debug};
use std::net::{Ipv4Addr, Ipv6Addr};

use std::{borrow::Cow, net::IpAddr};

use crate::error::{Result, StartupError};

/// Trait used to accept multiple types for the address of a server.
/// Default implementations are provided for `Ipv4Addr`, `String`, `&String` and `&str`.
pub trait ToHostAddress {
    /// Convert the type to an `Ipv4Addr`.
    fn to_address(&self) -> Result<IpAddr>;
}

impl ToHostAddress for Ipv4Addr {
    fn to_address(&self) -> Result<IpAddr> {
        Ok((*self).into())
    }
}

impl ToHostAddress for Ipv6Addr {
    fn to_address(&self) -> Result<IpAddr> {
        Ok((*self).into())
    }
}

impl ToHostAddress for [u8; 4] {
    fn to_address(&self) -> Result<IpAddr> {
        Ok(Ipv4Addr::new(self[0], self[1], self[2], self[3]).into())
    }
}

impl ToHostAddress for [u16; 8] {
    fn to_address(&self) -> Result<IpAddr> {
        Ok(Ipv6Addr::from(*self).into())
    }
}

impl ToHostAddress for [u8; 16] {
    fn to_address(&self) -> Result<IpAddr> {
        Ok(Ipv6Addr::from(*self).into())
    }
}

impl ToHostAddress for String {
    fn to_address(&self) -> Result<IpAddr> {
        Ok(Ipv4Addr::from(parse_ip(self)?).into())
    }
}

impl ToHostAddress for &String {
    fn to_address(&self) -> Result<IpAddr> {
        Ok(Ipv4Addr::from(parse_ip(self)?).into())
    }
}

impl ToHostAddress for &str {
    fn to_address(&self) -> Result<IpAddr> {
        Ok(Ipv4Addr::from(parse_ip(self)?).into())
    }
}

/// Parse a string to an IP address.
/// Will return a [`StartupError::InvalidIp`] if the IP has an invalid format.
/// Note: **Only IPv4 is supported**.
pub fn parse_ip(raw: &str) -> Result<[u8; 4]> {
    if raw == "localhost" {
        return Ok([127, 0, 0, 1]);
    }

    let mut ip = [0; 4];
    let mut split_ip = raw.split('.');

    for i in &mut ip {
        *i = split_ip
            .next()
            .and_then(|x| x.parse::<u8>().ok())
            .ok_or(StartupError::InvalidIp)?;
    }

    Ok(ip)
}

/// Filter out all `\r` and `\n` from a string.
/// This is to prevent [CRLF injection](https://datatracker.ietf.org/doc/html/rfc7230#section-9.4).
pub fn filter_crlf(value: &str) -> String {
    value.replace(['\r', '\n'], "")
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

/// Get the current time since the Unix Epoch.
/// Will panic if the system time is before the Unix Epoch.
#[cfg(feature = "extensions")]
pub(crate) fn epoch() -> std::time::Duration {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time is before the Unix Epoch. Make sure your date is set correctly.")
}

#[cfg(test)]
mod test {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    use super::{parse_ip, ToHostAddress};
    use crate::error::StartupError;

    #[test]
    fn test_parse_ip() {
        assert_eq!(parse_ip("123.231.43.3").unwrap(), [123, 231, 43, 3]);
        assert_eq!(parse_ip("123.231.43"), Err(StartupError::InvalidIp.into()));
        assert_eq!(
            parse_ip("256.231.43.3"),
            Err(StartupError::InvalidIp.into())
        );
    }

    #[test]
    fn test_from_ipv4_addr() {
        assert_eq!(
            Ipv4Addr::new(127, 0, 0, 1).to_address().unwrap(),
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
        );
    }

    #[test]
    fn test_from_ipv6_addr() {
        assert_eq!(
            Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1).to_address().unwrap(),
            IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))
        );
    }

    #[test]
    fn test_from_u8x4_addr() {
        assert_eq!(
            [127, 0, 0, 1].to_address().unwrap(),
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
        );
    }

    #[test]
    fn test_from_u16x8_addr() {
        assert_eq!(
            [0, 0, 0, 0, 0, 0, 0, 1].to_address().unwrap(),
            IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))
        );
    }

    #[test]
    fn test_from_u8x16_addr() {
        assert_eq!(
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
                .to_address()
                .unwrap(),
            IpAddr::V6(Ipv6Addr::from([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1
            ]))
        );
    }

    #[test]
    fn test_from_string_addr() {
        assert_eq!(
            "127.0.0.1".to_owned().to_address().unwrap(),
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
        );
    }

    #[test]
    fn test_from_ref_string_addr() {
        assert_eq!(
            (&"127.0.0.1".to_owned()).to_address().unwrap(),
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
        );
    }

    #[test]
    fn test_from_ref_str_addr() {
        assert_eq!(
            "127.0.0.1".to_address().unwrap(),
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
        );
    }
}
