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
    /// Convert a string to a method.
    ///
    /// If the string is not a valid method, `Method::CUSTOM(s)` is returned.
    /// ## Examples
    /// ```rust
    /// use afire::{Method};
    ///
    /// assert!(Method::from_string("GET") == Method::GET);
    /// assert!(Method::from_string("POST") == Method::POST);
    /// assert!(Method::from_string("PUT") == Method::PUT);
    /// assert!(Method::from_string("DELETE") == Method::DELETE);
    /// assert!(Method::from_string("OPTIONS") == Method::OPTIONS);
    /// assert!(Method::from_string("HEAD") == Method::HEAD);
    /// assert!(Method::from_string("PATCH") == Method::PATCH);
    /// assert!(Method::from_string("TRACE") == Method::TRACE);
    /// assert!(Method::from_string("foo") == Method::CUSTOM("FOO".to_string()));
    /// ```
    pub fn from_string(s: &str) -> Method {
        let upper_s = s.to_string().to_uppercase();
        match &upper_s[..] {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "OPTIONS" => Method::OPTIONS,
            "HEAD" => Method::HEAD,
            "PATCH" => Method::PATCH,
            "TRACE" => Method::TRACE,
            "ANY" => Method::ANY,
            _ => Method::CUSTOM(upper_s),
        }
    }
}

impl fmt::Display for Method {
    /// Returns the string representation of the method.
    ///
    /// ```rust
    /// use afire::{Method};
    ///
    /// assert_eq!("GET", Method::GET.to_string());
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Method::GET => write!(f, "GET"),
            Method::POST => write!(f, "POST"),
            Method::PUT => write!(f, "PUT"),
            Method::DELETE => write!(f, "DELETE"),
            Method::OPTIONS => write!(f, "OPTIONS"),
            Method::HEAD => write!(f, "HEAD"),
            Method::PATCH => write!(f, "PATCH"),
            Method::TRACE => write!(f, "TRACE"),
            Method::CUSTOM(ref s) => write!(f, "CUSTOM({})", s),
            Method::ANY => write!(f, "ANY"),
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
