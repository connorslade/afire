#[cfg(feature = "cookies")]
use super::cookie::SetCookie;
use super::header::Header;

/// Http Response
pub struct Response {
    /// Response status code
    pub status: u16,

    /// Response Data as Bytes
    pub data: Vec<u8>,

    /// Response Headers
    pub headers: Vec<Header>,
}

impl Response {
    /// Quick and easy way to create a response.
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Response, Header};
    /// // Create Response
    /// let response = Response::new(200, "Hello World", vec![Header::new("Content-Type", "text/plain")]);
    /// ```
    pub fn new(status: u16, data: &str, headers: Vec<Header>) -> Response {
        Response {
            status,
            data: data.as_bytes().to_vec(),
            headers,
        }
    }

    /// Create a response from a byte Vec.
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Response, Header};
    ///
    /// // Create Response
    /// let mut response = Response::new_raw(200, vec![72, 101, 108, 108, 111], vec![Header::new("Content-Type", "text/plain")]);
    /// ```
    pub fn new_raw(status: u16, data: Vec<u8>, headers: Vec<Header>) -> Response {
        Response {
            status,
            data,
            headers,
        }
    }

    /// Easy way to create a successful response.
    ///
    /// Will just pass status code 200.
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::Response;
    ///
    /// // Create Response
    /// let response = Response::new(200, "🍦", vec![]);
    /// let response2 = Response::ok("🍦", None);
    /// assert!(response == response2);
    /// ```
    pub fn ok(data: &str, headers: Option<Vec<Header>>) -> Response {
        Response::new(200, data, headers.unwrap_or_default())
    }

    /// Add a cookie to a response.
    /// ## Example
    /// ```
    /// // Import Library
    /// use afire::{Response, SetCookie};
    ///
    /// // Create Response and add cookie
    /// let response = Response::new(200, "🍦", vec![])
    ///     .add_cookie(SetCookie::new("name", "value"));
    /// ```
    #[cfg(feature = "cookies")]
    pub fn add_cookie(&self, cookie: SetCookie) -> Response {
        let mut new = self.clone();
        new.headers
            .push(Header::new("Set-Cookie", &cookie.to_string()));
        new
    }

    /// Add a vec of cookies to a response.
    /// ## Example
    /// ```
    /// // Import Library
    /// use afire::{Response, SetCookie};
    ///
    /// // Create Response and add cookie
    /// let response = Response::new(200, "🍦", vec![])
    ///     .add_cookies(vec![SetCookie::new("name", "value")]);
    /// ```
    #[cfg(feature = "cookies")]
    pub fn add_cookies(&self, cookie: Vec<SetCookie>) -> Response {
        for c in cookie {
            self.add_cookie(c);
        }

        self.clone()
    }
}

// Impl Clone for Response
impl Clone for Response {
    fn clone(&self) -> Response {
        Response {
            status: self.status,
            data: self.data.clone(),
            headers: self.headers.clone(),
        }
    }
}

impl PartialEq for Response {
    /// Allow comparing Responses
    fn eq(&self, other: &Self) -> bool {
        self.status == other.status && self.data == other.data && self.headers == other.headers
    }
}
