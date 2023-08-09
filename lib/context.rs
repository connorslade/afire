use std::{
    cell::RefCell,
    fmt::Display,
    io::Read,
    rc::Rc,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc, Barrier, Mutex,
    },
};

use crate::{
    error::Result,
    internal::sync::{ForceLockMutex, ForceLockRwLock, SingleBarrier},
    response::ResponseBody,
    Header, HeaderType, Request, Response, Server, Status,
};

pub struct Context<State: 'static + Send + Sync> {
    /// Reference to the server.
    pub server: Arc<Server<State>>,
    /// The request you are handling.
    pub req: Arc<Request>,
    /// The response you are building.
    pub(crate) response: Mutex<Response>,
    /// Various bit-packed flags.
    pub(crate) flags: ContextFlags,
}

pub(crate) struct ContextFlags(AtomicU8);

/// Flags that can be set on the [`Context`].
pub(crate) enum ContextFlag {
    /// The response has already been sent.
    ResponseSent = 1 << 1,
    // TODO: autosend on dirty?
    /// The response has been modified.
    ResponseDirty = 1 << 2,
    /// The user has guaranteed that the response will be sent.
    /// We should wait until the response is sent before continuing..
    /// This is different from the default behavior when a response is not sent of sending a 501 Not Implemented.
    GuaranteedSend = 1 << 3,
}

impl<State: 'static + Send + Sync> Context<State> {
    pub(crate) fn new(server: Arc<Server<State>>, req: Arc<Request>) -> Self {
        req.socket.add_barrier();
        Self {
            server,
            req,
            response: Mutex::new(Response::new()),
            flags: ContextFlags::new(),
        }
    }

    pub(crate) fn reset(&self) {
        self.flags.reset();
        *self.response.force_lock() = Response::new();
    }

    pub fn send(&self) -> Result<()> {
        self.response
            .force_lock()
            .write(self.req.socket.clone(), &self.server.default_headers)?;
        self.flags.set(ContextFlag::ResponseSent);

        if self.flags.get(ContextFlag::GuaranteedSend) {
            self.req.socket.unlock();
        }

        Ok(())
    }

    pub fn guarantee_will_send(&self) -> &Self {
        self.flags.set(ContextFlag::GuaranteedSend);
        self
    }
}

impl<State: 'static + Send + Sync> Context<State> {
    pub fn status(&self, code: impl Into<Status>) -> &Self {
        self.response.force_lock().status = code.into();
        self.flags.set(ContextFlag::ResponseDirty);
        self
    }

    pub fn reason(&self, reason: impl AsRef<str>) -> &Self {
        self.response.force_lock().reason = Some(reason.as_ref().to_owned());
        self.flags.set(ContextFlag::ResponseDirty);
        self
    }

    pub fn text(&self, text: impl Display) -> &Self {
        self.response.force_lock().data = text.to_string().as_bytes().to_vec().into();
        self.flags.set(ContextFlag::ResponseDirty);
        self
    }

    pub fn bytes(&self, bytes: impl Into<Vec<u8>>) -> &Self {
        self.response.force_lock().data = bytes.into().into();
        self.flags.set(ContextFlag::ResponseDirty);
        self
    }

    pub fn stream(&self, stream: impl Read + Send + 'static) -> &Self {
        self.response.force_lock().data = ResponseBody::Stream(Box::new(RefCell::new(stream)));
        self.flags.set(ContextFlag::ResponseDirty);
        self
    }

    pub fn header(&self, key: impl Into<HeaderType>, value: impl AsRef<str>) -> &Self {
        self.response
            .force_lock()
            .headers
            .push(Header::new(key, value));
        self.flags.set(ContextFlag::ResponseDirty);
        self
    }
}

impl ContextFlags {
    fn new() -> Self {
        Self(AtomicU8::new(0))
    }

    pub(crate) fn get(&self, flag: ContextFlag) -> bool {
        self.0.load(Ordering::Relaxed) & flag as u8 != 0
    }

    fn set(&self, flag: ContextFlag) {
        self.0.fetch_or(flag as u8, Ordering::Relaxed);
    }

    fn reset(&self) {
        self.0.store(0, Ordering::Relaxed);
    }
}
