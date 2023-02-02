//! Middleware is code that runs before and after the routes.
//! They can be used to Log Requests, Ratelimit Requests, add Analytics, etc.

use std::{any::type_name, sync::Arc};

use crate::{error::Result, Request, Response, Server};

/// A response from a middleware handler
pub enum MiddleResult {
    /// Continue to the next middleware
    Continue,
    /// Return a response and stop the middleware chain
    Abort(Response),
}

/// Middleware
pub trait Middleware {
    // /// Middleware to run before the raw request bytes are parsed
    //TODO: this
    // fn pre_raw(&self, _req: &mut Vec<u8>) -> MiddleResult {
    //     MiddleResult::Continue
    // }

    /// Middleware to run before routes.
    /// Because this is the `raw` version of [`pre`], it is passed a [`Result`].
    /// The default implementation calls [`pre`] if the [`Result`] is [`Ok`].
    fn pre_raw(&self, req: Result<&mut Request>) -> MiddleResult {
        if let Ok(req) = req {
            return self.pre(req);
        }
        MiddleResult::Continue
    }

    /// Middleware to run Before Routes
    fn pre(&self, _req: &mut Request) -> MiddleResult {
        MiddleResult::Continue
    }

    /// Middleware to run after routes.
    /// Because this is the `raw` version of [`post`], it is passed a [`Result`].
    /// The default implementation calls [`post`] if the [`Result`] is [`Ok`].
    fn post_raw(&self, req: Result<Arc<Request>>, res: Result<&mut Response>) -> MiddleResult {
        if let (Ok(req), Ok(res)) = (req, res) {
            return self.post(&*req, res);
        }
        MiddleResult::Continue
    }

    /// Middleware to run After Routes
    fn post(&self, _req: &Request, _res: &mut Response) -> MiddleResult {
        MiddleResult::Continue
    }

    /// Middleware to run after the response has been handled.
    /// Because this is the `raw` version of [`end`], it is passed a [`Result`].
    /// The default implementation calls [`end`] if the [`Result`] is [`Ok`].
    fn end_raw(&self, req: Result<&Request>, res: Result<&Response>) {
        if let (Ok(req), Ok(res)) = (req, res) {
            self.end(req, res);
        }
    }

    /// Middleware ot run after the response has been handled
    fn end(&self, _req: &Request, _res: &Response) {}

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
