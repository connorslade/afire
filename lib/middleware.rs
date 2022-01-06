use std::cell::RefCell;

use crate::{Request, Response, Server};

pub trait Middleware {
    fn pre(&mut self, req: Request) -> Request {
        req
    }

    fn post(&mut self, res: Response) -> Response {
        res
    }

    fn attach(self, server: &mut Server)
    where
        Self: Sized + 'static,
    {
        server.middleware.push(Box::new(RefCell::new(self)));
    }
}
