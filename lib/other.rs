use std::fmt;

use super::header::*;

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

/// Defines a route.
///
/// You should not use this directly.
/// It will be created automatically when useing server.get / post / put / delete / etc.
pub struct Route {
    pub(super) method: Method,
    pub(super) path: String,
    pub(super) handler: fn(Request) -> Response,
}

/// Http Request
pub struct Request {
    pub method: Method,
    pub path: String,
    pub headers: Vec<Header>,
    pub body: String,
}

/// Http Response
pub struct Response {
    pub status: u16,
    pub data: String,
    pub headers: Vec<Header>,
}

impl Response {
    /// Quick and easy way to create a response.
    pub fn new(status: u16, data: &str, headers: Vec<Header>) -> Response {
        Response {
            status,
            data: data.to_string(),
            headers: headers,
        }
    }
}

impl Request {
    /// Quick and easy way to create a request.
    ///
    /// ```rust
    /// use afire::{Request, Method};
    ///
    /// let request = Request {
    ///    method: Method::GET,
    ///    path: "/".to_string(),
    ///    headers: vec![],
    ///    body: "".to_string(),
    /// };
    ///
    /// assert!(request.compare(&Request::new(Method::GET, "/", vec![], "".to_string())));
    /// ```
    pub fn new(method: Method, path: &str, headers: Vec<Header>, body: String) -> Request {
        Request {
            method,
            path: path.to_string(),
            headers,
            body,
        }
    }

    pub fn compare(&self, other: &Request) -> bool {
        self.method == other.method
            && self.path == other.path
            && cmp_vec(&self.headers, &other.headers)
            && self.body == other.body
    }
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

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut headers = String::new();

        for header in &self.headers {
            headers.push_str(&format!("{}, ", header.to_string()));
        }

        f.debug_struct("Request")
            .field("method", &self.method)
            .field("path", &self.path)
            .field("headers", &headers)
            .field("body", &self.body)
            .finish()
    }
}

impl PartialEq for Method {
    /// Allow compatring Method Enums
    ///
    /// EX: Method::GET == Method::GET
    ///
    /// > True
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

/// Compares two Vectors
pub(crate) fn cmp_vec<T: std::cmp::PartialEq>(vec: &Vec<T>, vec2: &Vec<T>) -> bool {
    if vec.len() != vec2.len() {
        return false;
    }

    for i in 0..vec.len() {
        if vec[i] != vec2[i] {
            return false;
        }
    }
    return true;
}
