use std::{fmt, str::FromStr};

/// Methods for a request
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
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

    /// For routes that run on all methods
    ///
    /// Will not be use in a request
    ANY,
}

impl FromStr for Method {
    type Err = ();

    /// Convert a string to a method.
    ///
    /// If the string is not a valid method, `Method::CUSTOM(s)` is returned.
    /// ## Examples
    /// ```rust
    /// use std::str::FromStr;
    /// use afire::{Method};
    ///
    /// assert!(Method::from_str("GET").unwrap() == Method::GET);
    /// assert!(Method::from_str("POST").unwrap() == Method::POST);
    /// assert!(Method::from_str("PUT").unwrap() == Method::PUT);
    /// assert!(Method::from_str("DELETE").unwrap() == Method::DELETE);
    /// assert!(Method::from_str("OPTIONS").unwrap() == Method::OPTIONS);
    /// assert!(Method::from_str("HEAD").unwrap() == Method::HEAD);
    /// assert!(Method::from_str("PATCH").unwrap() == Method::PATCH);
    /// assert!(Method::from_str("TRACE").unwrap() == Method::TRACE);
    /// assert!(Method::from_str("foo") == Err(()));
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "DELETE" => Ok(Method::DELETE),
            "OPTIONS" => Ok(Method::OPTIONS),
            "HEAD" => Ok(Method::HEAD),
            "PATCH" => Ok(Method::PATCH),
            "TRACE" => Ok(Method::TRACE),
            _ => Err(()),
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
            Method::ANY => write!(f, "ANY"),
        }
    }
}
