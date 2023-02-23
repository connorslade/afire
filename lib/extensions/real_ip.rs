use std::net::IpAddr;

use crate::{HeaderType, Request};

/// Trait that adds methods for getting the real IP of a client through a reverse proxy.
/// If you are using the "X-Forwarded-For" header you can use `req.real_ip()` but if you are using a different header you will have to use `req.real_ip_header(...)`.
pub trait RealIp {
    /// Uses [`RealIp::real_ip_header`] with the "X-Forwarded-For" header.
    /// ## Example
    /// ```rust
    /// use afire::extension::RealIp;
    /// # use afire::{Server, Method, Response};
    ///
    /// # fn test(server: &mut Server) {
    /// server.route(Method::GET, "/", |req| {
    ///     let ip = req.real_ip();
    ///     Response::new().text(format!("Hello, {ip}"))
    /// });
    /// # }
    /// ```
    fn real_ip(&self) -> IpAddr {
        self.real_ip_header("X-Forwarded-For")
    }
    /// Gets the 'real IP' of a client by parsing the value of `header` into an IpAddr.
    /// If the connection is not coming from localhost, the header isn't found or the header contains an invalid IP address, the raw socket address will be returned.
    ///
    /// **Warning**: Make sure your reverse proxy is overwriting the specified header on the incoming requests so clients cant spoof their original Ips.
    fn real_ip_header(&self, header: impl Into<HeaderType>) -> IpAddr;
}

impl RealIp for Request {
    fn real_ip_header(&self, header: impl Into<HeaderType>) -> IpAddr {
        let ip = self.address.ip();

        // If the connection is not coming from localhost (likely from reverse proxy) return the raw IP
        if !ip.is_loopback() {
            return ip;
        }

        // If the 'X-Forwarded-For' Header is present and its value is a valid IP, return it
        // Otherwise return the socket address
        self.headers
            .get(header.into())
            .map(|x| x.parse().ok())
            .flatten()
            .unwrap_or(ip)
    }
}
