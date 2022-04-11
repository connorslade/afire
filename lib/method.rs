use std::fmt;

/// Methods for a request
#[derive(Hash, PartialEq, Eq)]
pub enum Method {
    /// GET Method
    ///
    /// Used for retrieving data
    GET,

    /// POST Method
    ///
    /// Used for submitting data
    POST,

    /// PUT Method
    ///
    /// Used for updating data
    PUT,

    /// DELETE Method
    ///
    /// Used for deleting data
    DELETE,

    /// OPTIONS Method
    ///
    /// Used for requesting information about the server
    OPTIONS,

    /// HEAD Method
    ///
    /// For getting the response from a GET request without the body
    HEAD,

    /// PATCH Method
    ///
    /// Used for applying a partial update to a resource
    PATCH,

    /// TRACE Method
    ///
    /// Used for tracing the route of a request
    TRACE,

    /// Custom Method
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
    pub fn from_string<T>(s: T) -> Method
    where
        T: AsRef<str>,
    {
        let upper_s = s.as_ref().to_uppercase();
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

// Impl Clone for Method
impl Clone for Method {
    fn clone(&self) -> Method {
        match *self {
            Method::GET => Method::GET,
            Method::POST => Method::POST,
            Method::PUT => Method::PUT,
            Method::DELETE => Method::DELETE,
            Method::OPTIONS => Method::OPTIONS,
            Method::HEAD => Method::HEAD,
            Method::PATCH => Method::PATCH,
            Method::TRACE => Method::TRACE,
            Method::CUSTOM(ref s) => Method::CUSTOM(s.clone()),
            Method::ANY => Method::ANY,
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
