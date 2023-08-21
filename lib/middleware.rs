//! Middleware is code that runs before and after the routes.
//! They can be used to Log Requests, Ratelimit Requests, add Analytics, etc.
//! For more information, see the [Middleware Example](https://github.com/Basicprogrammer10/afire/blob/main/examples/basic/middleware.rs).

use std::{any::type_name, sync::Arc};

use crate::{error::Result, trace::emoji, Request, Response, Server};

/// A response from a middleware handler
pub enum MiddleResult {
    /// Continue to the next middleware
    Continue,
    /// Stop the middleware chain
    Abort,
    /// Stop the middleware chain and send this response
    Send(Response),
}

/// Trait used to implement Middleware, which is code that runs before and after the routes - potentially modifying the request and response.
/// You can use Middleware to Log Requests, Ratelimit Requests, add Analytics, etc.
///
/// There are two types of hooks: raw and non-raw.
/// The raw hooks are passed a [`Result`], and their default implementation calls the non-raw hooks if the Result is Ok.
/// This allows you to handle errors (like page not found), while maintaining a clean API for middleware that doesn't need to handle errors.
///
/// ## Hooks
/// - [`Middleware::pre_raw`]
/// - [`Middleware::pre`]
/// - [`Middleware::post_raw`]
/// - [`Middleware::post`]
/// - [`Middleware::end_raw`]
/// - [`Middleware::end`]
///
pub trait Middleware {
    /// Middleware to run before routes.
    /// Because this is the `raw` version of [`Middleware::pre`], it is passed a [`Result`].
    /// The default implementation calls [`Middleware::pre`] if the [`Result`] is [`Ok`].
    fn pre_raw(&self, req: &mut Result<Request>) -> MiddleResult {
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
    /// Because this is the `raw` version of [`Middleware::post`], it is passed a [`Result`].
    /// The default implementation calls [`Middleware::post`] if the [`Result`] is [`Ok`].
    fn post_raw(&self, req: Result<Arc<Request>>, res: &mut Result<Response>) -> MiddleResult {
        if let (Ok(req), Ok(res)) = (req, res) {
            return self.post(&req, res);
        }
        MiddleResult::Continue
    }

    /// Middleware to run After Routes
    fn post(&self, _req: &Request, _res: &mut Response) -> MiddleResult {
        MiddleResult::Continue
    }

    /// Middleware to run after the response has been handled.
    /// Because this is the `raw` version of [`Middleware::end`], it is passed a [`Result`].
    /// The default implementation calls [`Middleware::end`] if the [`Result`] is [`Ok`].
    fn end_raw(&self, req: Result<Arc<Request>>, res: &Response) {
        if let Ok(req) = req {
            self.end(req, res)
        }
    }

    /// Middleware ot run after the response has been handled
    fn end(&self, _req: Arc<Request>, _res: &Response) {}

    /// Attach Middleware to a Server.
    /// If you want to get a reference to the server's state in your middleware state, you should override this method.
    fn attach<State>(self, server: &mut Server<State>)
    where
        Self: 'static + Send + Sync + Sized,
        State: 'static + Send + Sync,
    {
        trace!("{}Adding Middleware {}", emoji("📦"), type_name::<Self>());

        server.middleware.push(Box::new(self));
    }
}
