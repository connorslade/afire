use std::rc::Rc;

use crate::{path::Path, Context, Method, Request, error::AnyResult};

type Handler<State> = Box<dyn Fn(&Context<State>) -> AnyResult<()> + 'static + Send + Sync>;

/// Defines a route.
///
/// You should not use this directly.
/// It will be created automatically when using [`crate::Server::route`] or [`crate::Server::stateful_route`].
pub struct Route<State: 'static + Send + Sync> {
    /// Route Method (GET, POST, ANY, etc.)
    method: Method,

    /// Route path, in its tokenized form.
    path: Path,

    /// Route Handler, either stateless or stateful.
    pub(crate) handler: Handler<State>,
}

impl<State: 'static + Send + Sync> Route<State> {
    /// Creates a new route.
    pub(crate) fn new(method: Method, path: String, handler: Handler<State>) -> Self {
        Self {
            method,
            path: Path::new(path),
            handler,
        }
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
