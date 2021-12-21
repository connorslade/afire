use std::fmt;

use super::method::Method;
use super::request::Request;
use super::response::Response;
use crate::path::Path;

/// Defines a route.
///
/// You should not use this directly.
/// It will be created automatically when using server.route
// #[derive(Clone)]
pub struct Route {
    /// Route Method (GET, POST, ANY, etc)
    pub method: Method,

    /// Route Path
    pub path: Path,

    /// Route Handler
    pub handler: Box<dyn Fn(Request) -> Response>,
}

impl Route {
    /// Creates a new route.
    pub(crate) fn new(
        method: Method,
        path: String,
        handler: Box<dyn Fn(Request) -> Response>,
    ) -> Route {
        Route {
            method,
            path: Path::new(path),
            handler,
        }
    }
}

// TODO: Show handler in debug
impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Route")
            .field("method", &self.method)
            .field("path", &self.path)
            .finish()
    }
}
