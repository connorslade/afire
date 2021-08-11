use std::fmt;

/// Methods for a request
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    OPTIONS,
    HEAD,
    PATCH,
    TRACE,

    /// Custom request
    CUSTOM(String),

    /// For routes that run on all methods
    ///
    /// Will not be use in a request
    ANY,
}

impl Method {
    /// Returns the string representation of the method.
    ///
    /// ```rust
    /// use afire::{Method};
    ///
    /// assert_eq!("GET", Method::GET.to_string());
    /// ```
    pub fn to_string(&self) -> String {
        match self {
            Method::GET => "GET".to_string(),
            Method::POST => "POST".to_string(),
            Method::PUT => "PUT".to_string(),
            Method::DELETE => "DELETE".to_string(),
            Method::OPTIONS => "OPTIONS".to_string(),
            Method::HEAD => "HEAD".to_string(),
            Method::PATCH => "PATCH".to_string(),
            Method::TRACE => "TRACE".to_string(),
            Method::CUSTOM(t) => format!("CUSTOM({})", t),
            Method::ANY => "ANY".to_string(),
        }
    }
}

impl fmt::Debug for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Method")
            .field("method", &self.to_string())
            .finish()
    }
}

impl PartialEq for Method {
    /// Allow comparing Method Enums
    ///
    /// EX: Method::GET == Method::GET
    ///
    /// > True
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}
