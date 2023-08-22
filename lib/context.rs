use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Display,
    io::Read,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc, Mutex,
    },
};

use crate::{
    error::Result, internal::sync::ForceLockMutex, response::ResponseBody, Content, Header,
    HeaderType, Request, Response, Server, SetCookie, Status,
};

/// A collection of data important for handling a request.
/// It includes both the request data, and a reference to the server.
/// You also use it to build and send the response.
pub struct Context<State: 'static + Send + Sync> {
    /// Reference to the server.
    pub server: Arc<Server<State>>,
    /// The request you are handling.
    pub req: Arc<Request>,
    /// The path parameters.
    pub(crate) path_params: HashMap<String, String>,
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
    /// The response has been modified.
    ResponseDirty = 1 << 2,
    /// The user has guaranteed that the response will be sent.
    /// We should wait until the response is sent before continuing..
    /// This is different from the default behavior when a response is not sent of sending a 501 Not Implemented.
    GuaranteedSend = 1 << 3,
}

impl<State: 'static + Send + Sync> Context<State> {
    pub(crate) fn new(server: Arc<Server<State>>, req: Arc<Request>) -> Self {
        req.socket.reset_barrier();
        Self {
            server,
            req,
            path_params: HashMap::new(),
            response: Mutex::new(Response::new()),
            flags: ContextFlags::new(),
        }
    }

    /// Get a reference to the server's state.
    /// This is the same as `server.state.as_ref().unwrap()`, and as such it **will panic** if the server was not supplied a state.
    pub fn app(&self) -> &State {
        self.server.state.as_ref().unwrap()
    }

    /// Gets a path parameter.
    /// If the parameter does not exist, it **will panic**.
    pub fn param(&self, name: impl AsRef<str>) -> &String {
        self.path_params.get(name.as_ref()).unwrap()
    }

    /// Sends the response to the client.
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

    /// Guarantees that the response will be sent.
    /// This allows you to send the response after the handler has returned.
    pub fn guarantee_will_send(&self) -> &Self {
        self.flags.set(ContextFlag::GuaranteedSend);
        self
    }
}

impl<State: 'static + Send + Sync> Context<State> {
    // TODO: Maybe rename this?
    /// Overwrites the response with a new one.
    pub fn with_response(&self, res: Response) -> &Self {
        self.response.force_lock().data = res.data;
        self
    }

    /// Add a status code to a Response.
    /// This accepts [`Status`] as well as a [`u16`].
    pub fn status(&self, code: impl Into<Status>) -> &Self {
        self.response.force_lock().status = code.into();
        self.flags.set(ContextFlag::ResponseDirty);
        self
    }

    /// Manually set the Reason Phrase.
    /// If this is not set, it will be inferred from the status code.
    /// Non standard status codes will have a reason phrase of "OK".
    pub fn reason(&self, reason: impl AsRef<str>) -> &Self {
        self.response.force_lock().reason = Some(reason.as_ref().to_owned());
        self.flags.set(ContextFlag::ResponseDirty);
        self
    }

    /// Add text as data to a Response.
    /// Will accept any type that implements Display, such as [`String`], [`str`], [`i32`], serde_json::Value, etc.
    /// This response type is considered static and will be sent in one go, not chunked.
    pub fn text(&self, text: impl Display) -> &Self {
        self.response.force_lock().data = text.to_string().as_bytes().to_vec().into();
        self.flags.set(ContextFlag::ResponseDirty);
        self
    }

    /// Add raw bytes as data to a Response.
    /// This response type is considered static and will be sent in one go, not chunked.
    pub fn bytes(&self, bytes: impl Into<Vec<u8>>) -> &Self {
        self.response.force_lock().data = bytes.into().into();
        self.flags.set(ContextFlag::ResponseDirty);
        self
    }

    /// Add a stream as data to a Response.
    /// This response type is considered dynamic and will be streamed to the client in chunks using `Transfer-Encoding: chunked`.
    pub fn stream(&self, stream: impl Read + Send + 'static) -> &Self {
        self.response.force_lock().data = ResponseBody::Stream(Box::new(RefCell::new(stream)));
        self.flags.set(ContextFlag::ResponseDirty);
        self
    }

    /// Add a Header to a Response.
    /// Will accept any type that implements `AsRef<str>`, so [`String`], [`str`], [`&str`], etc.
    pub fn header(&self, key: impl Into<HeaderType>, value: impl AsRef<str>) -> &Self {
        self.response
            .force_lock()
            .headers
            .push(Header::new(key, value));
        self.flags.set(ContextFlag::ResponseDirty);
        self
    }

    /// Add a list of Headers to a Response.
    /// Only accepts a slice of [`Header`]s.
    pub fn headers(&self, headers: Vec<Header>) -> &Self {
        self.response.force_lock().headers.extend(headers);
        self.flags.set(ContextFlag::ResponseDirty);
        self
    }

    /// Add a cookie to a response.
    /// The [`SetCookie`] will be converted to a [`Header`] and added to the Response.
    pub fn cookie(&self, cookie: SetCookie) -> &Self {
        self.response
            .force_lock()
            .headers
            .push(Header::new("Set-Cookie", cookie.to_string()));
        self.flags.set(ContextFlag::ResponseDirty);
        self
    }

    /// Add a list of cookies to a response.
    pub fn cookies(&self, cookies: Vec<SetCookie>) -> &Self {
        self.response.force_lock().headers.extend(
            cookies
                .into_iter()
                .map(|cookie| Header::new("Set-Cookie", cookie.to_string())),
        );
        self.flags.set(ContextFlag::ResponseDirty);
        self
    }

    /// Set a Content Type on a Response with a [`Content`] enum.
    /// This will add a `Content-Type` header to the Response.
    pub fn content(&self, content_type: Content) -> &Self {
        self.response.force_lock().headers.push(content_type.into());
        self
    }

    /// Lets you modify the Response with a function before it is sent to the client.
    /// This can be used to have middleware that modifies the Response on specific routes.
    pub fn modifier(&self, modifier: impl FnOnce(&mut Response)) -> &Self {
        modifier(&mut self.response.force_lock());
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
}
