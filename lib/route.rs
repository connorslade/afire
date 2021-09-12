use std::fmt;

use super::method::Method;
use super::request::Request;
use super::response::Response;

/// Defines a route.
///
/// You should not use this directly.
/// It will be created automatically when using server.route
#[derive(Clone)]
pub struct Route {
    pub(super) method: Method,
    pub(super) path: String,
    pub(super) handler: fn(Request) -> Response,
}

impl Route {
    /// Creates a new route.
    pub(super) fn new(method: Method, path: String, handler: fn(Request) -> Response) -> Route {
        Route {
            method,
            path,
            handler,
        }
    }
}

impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Route")
            .field("method", &self.method)
            .field("path", &self.path)
            .field("handler", &self.handler)
            .finish()
    }
}
