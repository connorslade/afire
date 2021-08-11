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
    pub body: Vec<u8>,
}

/// Http Responce
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
    ///    body: vec![],
    /// };
    ///
    /// assert!(request.compare(&Request::new(Method::GET, "/", vec![], vec![])));
    /// ```
    pub fn new(method: Method, path: &str, headers: Vec<Header>, body: Vec<u8>) -> Request {
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
            && cmp_vec(&self.body, &other.body)
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
pub fn cmp_vec<T: std::cmp::PartialEq>(vec: &Vec<T>, vec2: &Vec<T>) -> bool {
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
