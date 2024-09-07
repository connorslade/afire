use std::{
    net::{IpAddr, SocketAddr},
    str,
    sync::{atomic::Ordering, Arc},
    thread,
};

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

/// Defines a server.
// todo: make not all this public
pub struct Server<State: 'static + Send + Sync> {
    /// Port to listen on.
    pub port: u16,

    /// Ip address to listen on.
    pub host: IpAddr,

    /// The event loop used to handle incoming connections.
    pub event_loop: Box<dyn EventLoop<State> + Send + Sync>,

    /// Routes to handle.
    pub routes: Vec<Route<State>>,

    // Other stuff
    /// Middleware
    pub middleware: Vec<Box<dyn Middleware + Send + Sync>>,

    /// Server wide App State
    pub state: Arc<State>,

    /// Default response for internal server errors
    pub error_handler: Box<dyn ErrorHandler<State> + Send + Sync>,

    /// The threadpool used for handling requests.
    /// You can also run your own tasks and resizes the threadpool.
    pub thread_pool: ThreadPool,

    pub config: Arc<ServerConfig>,
}

/// Implementations for Server
impl<State: Send + Sync> Server<State> {
    pub fn builder(host: impl ToHostAddress, port: u16, state: State) -> builder::Builder<State> {
        builder::Builder::new(host, port, state)
    }

    /// Starts the server with a threadpool.
    /// This is blocking.
    /// Will return an error if the server cant bind to the specified address, or of you are using stateful routes and have not set the state. (See [`Server::state`])
    ///
    /// ## Example
    /// ```rust,no_run
    /// # use afire::{Server, Response, Method, Content};
    /// // Creates a server on localhost (127.0.0.1) port 8080
    /// let mut server = Server::<()>::new("localhost", 8080);
    ///
    /// /* Define Routes, Attach Middleware, etc. */
    ///
    /// // Starts the server
    /// // This is blocking
    /// server.run().unwrap();
    /// ```
    pub fn run(self) -> Result<()> {
        let threads = self.thread_pool.threads();
        trace!(
            "{}Starting Server [{}:{}] ({} thread{})",
            emoji("✨"),
            self.host,
            self.port,
            threads,
            if threads == 1 { "" } else { "s" }
        );

        let addr = SocketAddr::new(self.host, self.port);
        let this = Arc::new(self);

        this.clone().event_loop.run(this, addr)?;

        trace!("{}Server Stopped", emoji("🛑"));
        Ok(())
    }

    /// Starts the server on a separate thread, retuning a handle that allows you to retrieve the server state and shutdown the server.
    pub fn run_async(self) -> Result<ServerHandle<State>> {
        let threads = self.thread_pool.threads();
        trace!(
            "{}Starting Server [{}:{}] ({} thread{})",
            emoji("✨"),
            self.host,
            self.port,
            threads,
            if threads == 1 { "" } else { "s" }
        );

        let handle = ServerHandle {
            config: self.config.clone(),
            state: self.state.clone(),
        };

        let addr = SocketAddr::new(self.host, self.port);
        let this = Arc::new(self);

        thread::spawn(move || {
            this.clone().event_loop.run(this, addr).unwrap();
        });

        Ok(handle)
    }

    /// Change the server's event loop.
    /// The default is [`TcpEventLoop`], which uses the standard library's built-in TCP listener.
    ///
    /// The [afire_tls](https://github.com/Basicprogrammer10/afire_tls) crate contains an event loop that uses rustls to handle TLS connections.
    pub fn event_loop(self, event_loop: impl EventLoop<State> + Send + Sync + 'static) -> Self {
        Server {
            event_loop: Box::new(event_loop),
            ..self
        }
    }

    /// Set the panic handler, which is called if a route or middleware panics.
    /// This is only available if the `panic_handler` feature is enabled.
    /// If you don't set it, the default response is 500 "Internal Server Error :/".
    /// Be sure that your panic handler wont panic, because that will just panic the whole application.
    /// ## Example
    /// ```rust
    /// # use afire::{Server, Response, Status, Context, route::RouteError};
    /// Server::<()>::new("localhost", 8080)
    ///     .error_handler(|ctx: &Context, err: RouteError| {
    ///         ctx.status(Status::InternalServerError)
    ///             .text(format!("Internal Server Error: {}", err.message))
    ///             .send()?;
    ///         Ok(())
    ///     });
    /// ```
    pub fn error_handler(self, res: impl ErrorHandler<State> + Send + Sync + 'static) -> Self {
        trace!("{}Setting Error Handler", emoji("✌"));

        Self {
            error_handler: Box::new(res),
            ..self
        }
    }

    /// Create a new route.
    /// The path can contain parameters, which are defined with `{...}`, as well as wildcards, which are defined with `*`.
    /// (`**` lets you math anything after the wildcard, including `/`)
    /// ## Example
    /// ```rust
    /// # use afire::{Server, Header, Method, Content};
    /// # let mut server = Server::<()>::new("localhost", 8080);
    /// // Define a route
    /// server.route(Method::GET, "/greet/{name}", |ctx| {
    ///     let name = ctx.param("name");
    ///
    ///     ctx.text(format!("Hello, {}!", name))
    ///         .content(Content::TXT)
    ///         .send()?;
    ///     Ok(())
    /// });
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
    /// # use afire::{Server, Response, Header, Method};
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<u32>::new("localhost", 8080).state(101);
    ///
    /// // Get its state and assert it is 101
    /// assert_eq!(*server.app(), 101);
    /// ```
    pub fn app(&self) -> &Arc<State> {
        &self.state
    }

    /// Schedule a shutdown of the server.
    /// Will complete all current requests before shutting down.
    pub fn shutdown(&self) {
        self.config.running.store(false, Ordering::Relaxed);
    }
}
