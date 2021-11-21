use std::fmt;
use std::panic::RefUnwindSafe;

use super::method::Method;
use super::request::Request;
use super::response::Response;

/// Defines a route.
///
/// You should not use this directly.
/// It will be created automatically when using server.route
// #[derive(Clone)]
pub struct Route {
    /// Route Methood (GET, POST, ANY, etc)
    pub method: Method,

    /// Route Path
    pub path: String,

    /// Route Handler
    pub handler: Box<dyn Fn(Request) -> Response + RefUnwindSafe>,
}

impl Route {
    /// Creates a new route.
    pub(super) fn new(
        method: Method,
        path: String,
        handler: Box<dyn Fn(Request) -> Response + RefUnwindSafe>,
    ) -> Route {
        let mut new_path = path;
        if new_path.chars().last().unwrap_or_default() == '/' {
            new_path.pop();
        }

        Route {
            method,
            path: new_path,
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
