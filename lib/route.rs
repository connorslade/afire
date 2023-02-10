use std::fmt::{self, Debug};
use std::rc::Rc;
use std::sync::Arc;

use crate::{path::Path, Method, Request, Response};

type StatelessRoute = Box<dyn Fn(&Request) -> Response + Send + Sync>;
type StatefulRoute<State> = Box<dyn Fn(Arc<State>, &Request) -> Response + Send + Sync>;

pub enum RouteType<State> {
    Stateless(StatelessRoute),
    Stateful(StatefulRoute<State>),
}

/// Defines a route.
///
/// You should not use this directly.
/// It will be created automatically when using [`crate::Server::route`] or [`crate::Server::stateful_route`].
#[derive(Debug)]
pub struct Route<State> {
    /// Route Method (GET, POST, ANY, etc.)
    method: Method,

    /// Route path, in its tokenized form.
    path: Path,

    /// Route Handler, either stateless or stateful.
    pub(crate) handler: RouteType<State>,
}

impl<State> Route<State> {
    /// Creates a new route.
    pub(crate) fn new(method: Method, path: String, handler: StatelessRoute) -> Self {
        Self {
            method,
            path: Path::new(path),
            handler: RouteType::Stateless(handler),
        }
    }

    /// Create a new stateful route
    pub(crate) fn new_stateful(
        method: Method,
        path: String,
        handler: StatefulRoute<State>,
    ) -> Self {
        Self {
            method,
            path: Path::new(path),
            handler: RouteType::Stateful(handler),
        }
    }

    /// Checks if the route is stateful.
    pub(crate) fn is_stateful(&self) -> bool {
        matches!(self.handler, RouteType::Stateful(_))
    }

    /// Checks if a Request matches the route.
    /// Returns the path parameters if it does.
    pub(crate) fn matches(&self, req: Rc<Request>) -> Option<Vec<(String, String)>> {
        if self.method != Method::ANY && self.method != req.method {
            return None;
        }
        self.path.match_path(req.path.clone())
    }
}

impl<State> Debug for RouteType<State> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RouteType::Stateless(_) => f.write_str("stateless"),
            RouteType::Stateful(_) => f.write_str("stateful"),
        }
    }
}
