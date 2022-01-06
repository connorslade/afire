use std::cell::RefCell;

use crate::{Request, Response, Server};

pub enum MiddleResponse {
    Continue,
    Add(Response),
    Send(Response),
}

pub enum MiddleRequest {
    Continue,
    Add(Request),
    Send(Response),
}

pub trait Middleware {
    /// Middleware to run Before Routes
    fn pre(&mut self, req: Request) -> MiddleRequest {
        MiddleRequest::Continue
    }

    /// Middleware to run After Routes
    fn post(&mut self, res: Response) -> MiddleResponse {
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
