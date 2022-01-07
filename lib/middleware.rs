use std::cell::RefCell;

use crate::{Request, Response, Server};

/// Middleware `post` Responses
pub enum MiddleResponse {
    /// Dont affect the Response
    Continue,

    /// Change the Response and continue to run Middleware (if any)
    Add(Response),

    /// Send Response immediately
    Send(Response),
}

/// Middleware `pre` Responses
///
/// Works with the Request
pub enum MiddleRequest {
    /// Dont affect the Request
    Continue,

    /// Change the Request and continue to run Middleware (if any) then routes
    Add(Request),

    /// Send a Response immediately
    Send(Response),
}

/// Middleware
pub trait Middleware {
    /// Middleware to run Before Routes
    fn pre(&mut self, _req: Request) -> MiddleRequest {
        MiddleRequest::Continue
    }

    /// Middleware to run After Routes
    fn post(&mut self, _res: Response) -> MiddleResponse {
        MiddleResponse::Continue
    }

    /// Attatch Middleware to a Server
    fn attach(self, server: &mut Server)
    where
        Self: Sized + 'static,
    {
        server.middleware.push(Box::new(RefCell::new(self)));
    }
}
