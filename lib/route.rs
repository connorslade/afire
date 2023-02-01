use std::fmt::{self, Debug};
use std::sync::Arc;

use crate::{path::Path, Method, Request, Response};

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
#[derive(Debug)]
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

    pub(crate) fn is_stateful(&self) -> bool {
        match self.handler {
            RouteType::Stateless(_) => false,
            RouteType::Statefull(_) => true,
        }
    }
}

impl<State> Debug for RouteType<State> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RouteType::Stateless(_) => f.write_str("stateless"),
            RouteType::Statefull(_) => f.write_str("statefull"),
        }
    }
}
