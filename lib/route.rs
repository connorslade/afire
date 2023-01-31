use std::fmt;
use std::sync::Arc;

use super::method::Method;
use super::request::Request;
use super::response::Response;
use crate::path::Path;

type StatelessRoute = Box<dyn Fn(&Request) -> Response + Send + Sync>;
type StatefullRoute<State> = Box<dyn Fn(Arc<State>, &Request) -> Response + Send + Sync>;

pub enum RouteType<State> {
    Stateless(StatelessRoute),
    Statefull(StatefullRoute<State>),
}

/// Defines a route.
///
/// You should not use this directly.
/// It will be created automatically when using server.route
pub struct Route<State> {
    /// Route Method (GET, POST, ANY, etc)
    pub method: Method,

    /// Route Path
    pub path: Path,

    /// Route Handler
    pub handler: RouteType<State>,
}

impl<State> Route<State> {
    /// Creates a new route.
    pub fn new(method: Method, path: String, handler: StatelessRoute) -> Self {
        Self {
            method,
            path: Path::new(path),
            handler: RouteType::Stateless(handler),
        }
    }

    /// Create a new stateful route
    pub fn new_stateful(method: Method, path: String, handler: StatefullRoute<State>) -> Self {
        Self {
            method,
            path: Path::new(path),
            handler: RouteType::Statefull(handler),
        }
    }
}

// TODO: Show handler in debug
impl<State> fmt::Debug for Route<State> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Route")
            .field("method", &self.method)
            .field("path", &self.path)
            .field(
                "handler",
                &match self.handler {
                    RouteType::Stateless(_) => "stateless",
                    RouteType::Statefull(_) => "statefull",
                },
            )
            .finish()
    }
}
