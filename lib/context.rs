use std::{
    rc::Rc,
    sync::{Arc, Barrier}, cell::RefCell,
};

use crate::{Request, Response, Server};

pub struct Context<State: 'static + Send + Sync> {
    pub server: Arc<Server<State>>,
    pub req: Rc<Request>,
    pub(crate) response: RefCell<Option<Response>>,
    req_barrier: Barrier,
}

impl<State: 'static + Send + Sync> Context<State> {
    pub(crate) fn new(server: Arc<Server<State>>, req: Rc<Request>) -> Self {
        Self {
            server,
            req,
            req_barrier: Barrier::new(2),
            response: RefCell::new(None),
        }
    }
}
