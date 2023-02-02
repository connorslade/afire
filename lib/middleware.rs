//! Middleware is code that runs before and after the routes.
//! They can be used to Log Requests, Ratelimit Requests, add Analytics, etc.

use std::any::type_name;

use crate::{Request, Response, Server};

pub enum MiddleResult {
    Continue,
    Abort(Response),
}

/// Middleware
pub trait Middleware {
    fn pre_raw(&self, _req: &mut Vec<u8>) -> MiddleResult {
        MiddleResult::Continue
    }

    /// Middleware to run Before Routes
    fn pre(&self, _req: &mut Request) -> MiddleResult {
        MiddleResult::Continue
    }

    /// Middleware to run After Routes
    fn post(&self, _req: &Request, _res: &mut Response) -> MiddleResult {
        MiddleResult::Continue
    }

    /// Middleware ot run after the response has been handled
    fn end(&self, _req: &Request, _res: &Response) {}

    // TODO: Error middleware?

    /// Attatch Middleware to a Server
    fn attach<State>(self, server: &mut Server<State>)
    where
        Self: 'static + Send + Sync + Sized,
        State: 'static + Send + Sync,
    {
        trace!("📦 Adding Middleware {}", type_name::<Self>());

        server.middleware.push(Box::new(self));
    }
}
