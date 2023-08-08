use std::{
    cell::RefCell,
    fmt::Display,
    io::Read,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, AtomicU8, Ordering},
        Arc, Mutex,
    },
};

use crate::{
    error::Result, internal::common::ForceLockMutex, response::ResponseBody, Header, HeaderType,
    Request, Response, Server, Status,
};

pub struct Context<State: 'static + Send + Sync> {
    pub server: Arc<Server<State>>,
    pub req: Rc<Request>,
    pub(crate) response: Mutex<Response>,
    pub(crate) response_dirty: AtomicBool,
}

impl<State: 'static + Send + Sync> Context<State> {
    pub(crate) fn new(server: Arc<Server<State>>, req: Rc<Request>) -> Self {
        Self {
            server,
            req,
            response: Mutex::new(Response::new()),
            response_dirty: AtomicBool::new(false),
        }
    }

    pub fn send(&self) -> Result<()> {
        self.response
            .force_lock()
            .write(self.req.socket.clone(), &self.server.default_headers)?;
        Ok(())
    }
}

impl<State: 'static + Send + Sync> Context<State> {
    pub fn status(&self, code: impl Into<Status>) -> &Self {
        self.response.force_lock().status = code.into();
        self
    }

    pub fn reason(&self, reason: impl AsRef<str>) -> &Self {
        self.response.force_lock().reason = Some(reason.as_ref().to_owned());
        self
    }

    pub fn text(&self, text: impl Display) -> &Self {
        self.response.force_lock().data = text.to_string().as_bytes().to_vec().into();
        self
    }

    pub fn bytes(&self, bytes: impl Into<Vec<u8>>) -> &Self {
        self.response.force_lock().data = bytes.into().into();
        self
    }

    pub fn stream(&self, stream: impl Read + Send + 'static) -> &Self {
        self.response.force_lock().data = ResponseBody::Stream(Box::new(RefCell::new(stream)));
        self
    }

    pub fn header(&self, key: impl Into<HeaderType>, value: impl AsRef<str>) -> &Self {
        self.response
            .force_lock()
            .headers
            .push(Header::new(key, value));
        self
    }
}
