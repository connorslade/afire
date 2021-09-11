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
    /// ## Examples
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
    /// ## Examples
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
    /// ## Examples
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

    #[cfg(feature = "cookies")]
    pub fn add_cookie(&mut self, cookie: SetCookie) {
        self.headers
            .push(Header::new("Set-Cookie", &cookie.to_string()));
    }

    #[cfg(feature = "cookies")]
    pub fn add_cookies(&mut self, cookie: Vec<SetCookie>) {
        for c in cookie {
            self.add_cookie(c);
        }
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
