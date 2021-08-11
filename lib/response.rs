use super::header::Header;

/// Http Response
pub struct Response {
    pub status: u16,
    pub data: String,
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
            data: data.to_string(),
            headers: headers,
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
        Response::new(200, data, headers.unwrap_or(vec![]))
    }
}

impl PartialEq for Response {
    /// Allow comparing Responses
    fn eq(&self, other: &Self) -> bool {
        self.status == other.status && self.data == other.data && self.headers == other.headers
    }
}
