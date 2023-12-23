use std::{
    cell::RefCell,
    fmt::Display,
    io::Read,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc, Mutex, MutexGuard,
    },
};

use crate::{
    error::{HandleError, Result},
    internal::{router::PathParameters, sync::ForceLockMutex},
    response::ResponseBody,
    Content, Header, Request, Response, Server, SetCookie, Status,
};

/// A collection of data important for handling a request.
/// It includes both the request data, and a reference to the server.
/// You also use it to build and send the response.
pub struct Context<State: 'static + Send + Sync = ()> {
    /// Reference to the server.
    pub server: Arc<Server<State>>,
    /// The request you are handling.
    pub req: Arc<Request>,
    /// The path parameters.
    pub(crate) path_params: Option<PathParameters>,
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
            path_params: None,
            response: Mutex::new(Response::new()),
            flags: ContextFlags::new(),
        }
    }

    /// Get a reference to the server's state.
    /// This is the same as `self.server.state.clone().unwrap()`, and as such it **will panic** if the server was not supplied a state.
    pub fn app(&self) -> Arc<State> {
        self.server
            .state
            .clone()
            .expect("Server does not have a state.")
    }

    /// Gets a path parameter by name.
    /// If the parameter does not exist, it **will panic**.
    /// Because any path parameters are guaranteed to exist if the route matches, there is no need to be able to check if a parameter exists.
    ///
    /// [`Context::param_idx`] is a more efficient way to get a path parameter.
    /// If those nanoseconds matter to you, in a *web server*, you can use that instead.
    ///
    /// ## Example
    /// ```
    /// # use afire::prelude::*;
    /// # fn test(server: &mut Server) {
    /// server.route(Method::GET, "/greet/{name}", |ctx| {
    ///     let name = ctx.param("name");
    ///     ctx.text(format!("Hello, {}!", name)).send()?;
    ///     Ok(())
    /// });
    /// # }
    pub fn param(&self, name: impl AsRef<str>) -> &str {
        let name = name.as_ref();
        let params = self.path_params.as_ref().unwrap();
        params
            .get(name, &self.req.path)
            .unwrap_or_else(|| panic!("Path parameter {} does not exist.", name))
    }

    /// Gets a path parameter by index.
    /// If the parameter does not exist, it **will panic**.
    /// Because any path parameters are guaranteed to exist if the route matches, there is no need to be able to check if a parameter exists.
    ///
    /// Because [`Context::param`] needs to linearly search for the parameter's name, this method is faster.
    /// You can even use this method if the parameter has a name.
    ///
    /// ## Example
    /// ```
    /// # use afire::prelude::*;
    /// # fn test(server: &mut Server) {
    /// server.route(Method::GET, "/greet/{}", |ctx| {
    ///     let name = ctx.param_idx(0);
    ///     ctx.text(format!("Hello, {}!", name)).send()?;
    ///     Ok(())
    /// });
    /// # }
    pub fn param_idx(&self, idx: usize) -> &str {
        let params = self.path_params.as_ref().unwrap();
        params
            .get_index(idx, &self.req.path)
            .unwrap_or_else(|| panic!("Path parameter #{} does not exist.", idx))
    }

    /// Gets a reference to the internal response.
    /// This is mostly useful for when you need to inspect the current state of the response, or overwrite it in an error handler.
    pub fn get_response(&self) -> MutexGuard<Response> {
        self.response.force_lock()
    }

    /// Sends the response to the client.
    /// This method must not be called more than once per request.
    ///
    /// ### Returning without sending a response
    ///
    /// If you return from the handler without sending a response, either because you forgot or because you are sending the response asynchronously from another thread, the server will by default send a 501 Not Implemented response.
    /// If you want to allow doing this, you can use [`Context::guarantee_will_send`] to signal to the server that you *will* send the response.
    /// This will cause the server to wait until the response is sent before looking for another another request.
    ///
    /// ### Sending multiple responses
    ///
    /// If you send multiple responses for a single request, all but the first will be ignored and log a warning with the [`Error` log level][`crate::trace::Level`].
    ///
    /// ## Example
    /// ```
    /// # use afire::prelude::*;
    /// # fn test(server: &mut Server) {
    /// server.route(Method::GET, "/", |ctx| {
    ///     ctx.text("Hello World!").send()?;
    ///    Ok(())
    /// });
    ///
    /// // A more compact way to send a response.
    /// server.route(Method::GET, "/", |ctx| Ok(ctx.text("Hello World!").send()?));
    /// # }
    pub fn send(&self) -> Result<()> {
        if self.flags.get(ContextFlag::ResponseSent) {
            return Err(HandleError::ResponseAlreadySent.into());
        }

        // TODO: NOT CALLING POST_RAW
        for i in &self.server.middleware {
            i.post(&self.req.clone(), &mut self.response.force_lock());
        }

        self.response
            .force_lock()
            .write(self.req.socket.clone(), &self.server.default_headers)?;
        self.flags.set(ContextFlag::ResponseSent);

        let res = self.response.force_lock();
        for i in self.server.middleware.iter() {
            i.end_raw(Ok(self.req.clone()), &res);
        }

        if self.flags.get(ContextFlag::GuaranteedSend) {
            self.req.socket.unlock();
        }

        Ok(())
    }

    /// Guarantees that the response will be sent.
    /// This allows you to send the response after the handler has returned.
    ///
    /// We send a not implemented response by default because otherwise the handler would hang if you forgot to send a response.
    /// ## Example
    /// ```
    /// # use afire::prelude::*;
    /// # use std::thread;
    /// # fn test(server: &mut Server) {
    /// server.route(Method::GET, "/", |ctx| {
    ///     // Tell the the router that we *will* send the response later.
    ///     ctx.guarantee_will_send();
    ///
    ///     // We can't take ctx into the thread so we clone the socket.
    ///     // This probably isn't something you would actually do.
    ///     // But for example, we do something similar in the websocket implementation.
    ///     let socket = ctx.req.socket.clone();
    ///     thread::spawn(move || {
    ///         Response::new()
    ///             .text("Hello from another thread")
    ///             .write(socket, &[])
    ///             .unwrap();
    ///     });
    ///     Ok(())
    /// });
    /// # }
    /// ```
    pub fn guarantee_will_send(&self) -> &Self {
        self.flags.set(ContextFlag::GuaranteedSend);
        self
    }
}

/// Response building methods.
/// These methods are the same as the ones on [`Response`], but they just mutate the internal response of the context.
/// Don't forget to call [`Context::send`] once you are done building the response.
/// ## Example
/// ```
/// # use afire::prelude::*;
/// # fn test(server: &mut Server) {
/// server.route(Method::GET, "/", |ctx| {
///     // Because the internal response is mutated,
///     // the sent response will have the "X-Test" header.
///     ctx.header(("X-Test", "Test"));
///     ctx.text("Hello World!").send()?;
///     Ok(())
/// });
/// # }
/// ```
impl<State: 'static + Send + Sync> Context<State> {
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
    /// Will accept any type that implements `Into<Header>`, so you can use a tuple of `(impl Into<HeaderName>, impl AsRef<str>)` like `(&str, &str)` or a [header struct][`crate::header`] (recommended).
    /// ## Example
    /// ```
    /// # use afire::prelude::*;
    /// # use afire::headers;
    /// # fn test(server: &mut Server) {
    /// server.route(Method::GET, "/", |ctx| {
    ///     ctx.header(("X-Test", "Test")); // Set 'X-Test' header to 'Test'
    ///     ctx.header(headers::Server::new("teapot")); // Set 'Server' header to 'teapot'
    ///
    ///     ctx.text("Hello World!").send()?;
    ///     Ok(())   
    /// });
    /// # }
    /// ```
    pub fn header(&self, header: impl Into<Header>) -> &Self {
        self.response.force_lock().headers.push(header.into());
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
    /// Create a new ContextFlags with no flags set.
    fn new() -> Self {
        Self(AtomicU8::new(0))
    }

    /// Get the value of a flag.
    pub(crate) fn get(&self, flag: ContextFlag) -> bool {
        self.0.load(Ordering::Relaxed) & flag as u8 != 0
    }

    /// Set a flag to true.
    fn set(&self, flag: ContextFlag) {
        self.0.fetch_or(flag as u8, Ordering::Relaxed);
    }
}
