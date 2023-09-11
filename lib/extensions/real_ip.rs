//! Methods for getting the real IP of a client through a reverse proxy.
//!
//! **Warning**: Make sure your reverse proxy is overwriting the specified header on the incoming requests so clients cant spoof their original Ips.

use std::net::{IpAddr, Ipv6Addr};

use crate::{HeaderName, Request};

/// Trait that adds methods for getting the real IP of a client through a reverse proxy.
/// If you are using the "X-Forwarded-For" header you can use `req.real_ip()` but if you are using a different header you will have to use `req.real_ip_header(...)`.
pub trait RealIp {
    /// Uses [`RealIp::real_ip_header`] with the ["X-Forwarded-For"](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Forwarded-For) header.
    /// ## Example
    /// ```rust
    /// use afire::extensions::RealIp;
    /// # use afire::{Server, Method, Response};
    ///
    /// # fn test(server: &mut Server) {
    /// server.route(Method::GET, "/", |ctx| {
    ///     let ip = ctx.req.real_ip();
    ///     ctx.text(format!("Hello, {ip}"))
    ///         .send()?;
    ///     Ok(())
    /// });
    /// # }
    /// ```
    fn real_ip(&self) -> IpAddr {
        self.real_ip_header(HeaderName::XForwardedFor)
    }

    /// Gets the 'real IP' of a client by parsing the value of `header` into an IpAddr.
    /// If the connection is not coming from localhost (    ), the header isn't found or the header contains an invalid IP address, the raw socket address will be returned.
    ///
    /// **Warning**: Make sure your reverse proxy is overwriting the specified header on the incoming requests so clients cant spoof their original Ips.
    fn real_ip_header(&self, header: impl Into<HeaderName>) -> IpAddr;
}

impl RealIp for Request {
    fn real_ip_header(&self, header: impl Into<HeaderName>) -> IpAddr {
        let ip = self.address.ip();

        // If the connection is not coming from localhost (likely from reverse proxy) return the raw IP
        if !is_local(ip) {
            return ip;
        }

        // If the 'X-Forwarded-For' Header is present and its value is a valid IP, return it
        // Otherwise return the socket address
        self.headers
            .get(header.into())
            .and_then(|x| x.parse().ok())
            .unwrap_or(ip)
    }
}

fn is_local(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => ip.is_loopback() || ip.octets()[0] == 172 || ip.is_private(),
        IpAddr::V6(ip) => ip.is_loopback() || ipv6_is_unique_local(ip),
    }
}

const fn ipv6_is_unique_local(ip: Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xfe00) == 0xfc00
}
