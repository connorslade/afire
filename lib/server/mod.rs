use std::{
    str,
    sync::{atomic::Ordering, Arc},
    thread,
};

use builder::Builder;
use config::ServerConfig;
use handle::ServerHandle;

use crate::{
    error::{AnyResult, Result},
    internal::{event_loop::EventLoop, misc::ToHostAddress, thread_pool::ThreadPool},
    route::{ErrorHandler, Route},
    trace::emoji,
    Context, Method, Middleware,
};

pub mod builder;
pub mod config;
pub mod handle;

/// A web server.
pub struct Server<State = ()>
where
    State: Send + Sync + 'static,
{
    /// Server configuration including:
    /// - Listening address
    /// - Default headers
    /// - Keep-alive
    /// - Socket timeout
    pub config: Arc<ServerConfig>,
    /// Server wide App State
    state: Arc<State>,
    /// The event loop used to handle incoming connections.
    event_loop: Box<dyn EventLoop<State> + Send + Sync>,
    /// Routes to handle.
    pub(crate) routes: Vec<Route<State>>,
    /// Middleware
    pub(crate) middleware: Vec<Box<dyn Middleware + Send + Sync>>,
    /// Default response for internal server errors
    pub(crate) error_handler: Box<dyn ErrorHandler<State> + Send + Sync>,
    /// The threadpool used for handling requests.
    /// You can also run your own tasks and resizes the threadpool.
    pub thread_pool: ThreadPool,
}

/// Implementations for Server
impl<State> Server<State>
where
    State: Send + Sync,
{
    /// Creates a new server builder with the specified host/port and state.
    /// If you don't want to use state, you can use `()` as the state.
    pub fn builder(host: impl ToHostAddress, port: u16, state: State) -> Builder<State> {
        Builder::new(host, port, state)
    }

    /// Starts the server with a threadpool.
    /// This is blocking.
    /// Will return an error if the server cant bind to the specified address, or of you are using stateful routes and have not set the state. (See [`Server::state`])
    ///
    /// ## Example
    /// ```rust,no_run
    /// # use afire::{Server, Response, Method, Content, error::Result};
    /// # fn run() -> Result<()> {
    /// // Creates a server on localhost (127.0.0.1) port 8080
    /// let mut server = Server::builder("localhost", 8080, ()).build()?;
    ///
    /// /* Define Routes, Attach Middleware, etc. */
    ///
    /// // Starts the server
    /// // This is blocking
    /// server.run()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn run(self) -> Result<()> {
        let threads = self.thread_pool.threads();
        trace!(
            "{}Starting Server [{}] ({} thread{})",
            emoji("✨"),
            self.config.host,
            threads,
            if threads == 1 { "" } else { "s" }
        );

        let host = self.config.host;
        let this = Arc::new(self);
        this.clone().event_loop.run(this, host)?;

        trace!("{}Server Stopped", emoji("🛑"));
        Ok(())
    }

    /// Starts the server on a separate thread, retuning a handle that allows you to retrieve the server state and shutdown the server.
    /// See [`Server::run`] and [`ServerHandle`] for more information.
    pub fn run_async(self) -> Result<ServerHandle<State>> {
        let threads = self.thread_pool.threads();
        trace!(
            "{}Starting Server [{}] ({} thread{})",
            emoji("✨"),
            self.config.host,
            threads,
            if threads == 1 { "" } else { "s" }
        );

        let handle = ServerHandle {
            config: self.config.clone(),
            state: self.state.clone(),
        };

        let host = self.config.host;
        let this = Arc::new(self);

        thread::spawn(move || {
            this.clone().event_loop.run(this, host).unwrap();
        });

        Ok(handle)
    }

    /// Create a new route.
    /// The path can contain parameters, which are defined with `{...}`, as well as wildcards, which are defined with `*`.
    /// (`**` lets you math anything after the wildcard, including `/`)
    /// ## Example
    /// ```rust
    /// # use afire::{Server, Header, Method, Content, error::Result};
    /// # fn run() -> Result<()> {
    /// # let mut server = Server::builder("localhost", 8080, ()).build()?;
    /// // Define a route
    /// server.route(Method::GET, "/greet/{name}", |ctx| {
    ///     let name = ctx.param("name");
    ///
    ///     ctx.text(format!("Hello, {}!", name))
    ///         .content(Content::TXT)
    ///         .send()?;
    ///     Ok(())
    /// });
    /// # Ok(())
    /// # }
    /// ```
    pub fn route(
        &mut self,
        method: Method,
        path: impl AsRef<str>,
        handler: impl Fn(&Context<State>) -> AnyResult<()> + Send + Sync + 'static,
    ) -> &mut Self {
        trace!("{}Adding Route {} {}", emoji("🚗"), method, path.as_ref());

        self.routes.push(
            Route::new(method, path.as_ref(), Box::new(handler)).expect("Error creating route."),
        );
        self
    }

    /// Gets a reference to the current server state set outside of stateful routes.
    /// Will <u>panic</u> if the server has no state.
    /// ## Example
    /// ```rust
    /// # use afire::{Server, Response, Header, Method, error::Result};
    /// # fn run() -> Result<()> {
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::builder("localhost", 8080, 101).build()?;
    ///
    /// // Get its state and assert it is 101
    /// assert_eq!(*server.app(), 101);
    /// # Ok(())
    /// # }
    /// ```
    pub fn app(&self) -> Arc<State> {
        self.state.clone()
    }

    /// Schedule a shutdown of the server.
    /// Will complete all current requests before shutting down.
    pub fn shutdown(&self) {
        self.config.running.store(false, Ordering::Relaxed);
    }
}
