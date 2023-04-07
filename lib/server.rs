// Import STD libraries
use std::any::type_name;
use std::net::{IpAddr, SocketAddr, TcpListener};
use std::rc::Rc;
use std::str;
use std::sync::Arc;
use std::time::Duration;

// Import local files
use crate::{
    error::Result, error::StartupError, handle::handle, header::Headers,
    internal::common::ToHostAddress, thread_pool::ThreadPool, Content, Header, HeaderType, Method,
    Middleware, Request, Response, Route, Status, VERSION,
};

type ErrorHandler<State> =
    Box<dyn Fn(Option<Arc<State>>, &Box<Result<Rc<Request>>>, String) -> Response + Send + Sync>;

/// Defines a server.
pub struct Server<State: 'static + Send + Sync = ()> {
    /// Port to listen on.
    pub port: u16,

    /// Ip address to listen on.
    pub ip: IpAddr,

    /// Routes to handle.
    pub routes: Vec<Route<State>>,

    // Other stuff
    /// Middleware
    pub middleware: Vec<Box<dyn Middleware + Send + Sync>>,

    /// Server wide App State
    pub state: Option<Arc<State>>,

    /// Default response for internal server errors
    pub error_handler: ErrorHandler<State>,

    /// Headers automatically added to every response.
    pub default_headers: Headers,

    /// Weather to allow keep-alive connections.
    /// If this is set to false, the server will close the connection after every request.
    /// This is enabled by default.
    pub keep_alive: bool,

    /// Socket Timeout
    pub socket_timeout: Option<Duration>,
}

/// Implementations for Server
impl<State: Send + Sync> Server<State> {
    /// Creates a new server on the specified address and port.
    /// `raw_ip` can be either an IP address or 'localhost', which expands to 127.0.0.1.
    ///
    /// ## Example
    /// ```rust
    /// # use afire::Server;
    /// // Create a server for localhost on port 8080
    /// // Note: The server has not been started yet
    /// let mut server = Server::<()>::new("localhost", 8080);
    /// ```
    pub fn new(raw_ip: impl ToHostAddress, port: u16) -> Self {
        trace!("🐍 Initializing Server v{}", VERSION);
        Server {
            port,
            ip: raw_ip.to_address().unwrap(),
            routes: Vec::new(),
            middleware: Vec::new(),

            error_handler: Box::new(|_state, _req, err| {
                Response::new()
                    .status(Status::InternalServerError)
                    .text(format!("Internal Server Error :/\nError: {err}"))
                    .content(Content::TXT)
            }),

            default_headers: Headers(vec![Header::new("Server", format!("afire/{VERSION}"))]),
            keep_alive: true,
            socket_timeout: None,
            state: None,
        }
    }

    /// Starts the server without a threadpool.
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
    /// server.start().unwrap();
    /// ```
    pub fn start(&self) -> Result<()> {
        trace!("✨ Starting Server [{}:{}]", self.ip, self.port);
        self.check()?;

        let listener = TcpListener::bind(SocketAddr::new(self.ip, self.port))?;

        for event in listener.incoming() {
            handle(event?, self);
        }

        // We should never get Here
        unreachable!()
    }

    /// Start the server with a threadpool of `threads` threads.
    /// Just like [`Server::start`], this is blocking.
    /// Will return an error if the server cant bind to the specified address, or of you are using stateful routes and have not set the state. (See [`Server::state`])
    ///
    /// ## Example
    /// ```rust,no_run
    /// // Import Library
    /// use afire::{Server, Response, Header, Method};
    ///
    /// // Creates a server on localhost (127.0.0.1) port 8080
    /// let mut server = Server::<()>::new("localhost", 8080);
    ///
    /// /* Define Routes, Attach Middleware, etc. */
    ///
    /// // Starts the server with 4 threads
    /// // This is blocking
    /// server.start_threaded(4).unwrap();
    /// ```
    pub fn start_threaded(self, threads: usize) -> Result<()> {
        trace!(
            "✨ Starting Server [{}:{}] ({} threads)",
            self.ip,
            self.port,
            threads
        );
        self.check()?;

        let listener = TcpListener::bind(SocketAddr::new(self.ip, self.port))?;
        let pool = ThreadPool::new(threads);
        let this = Arc::new(self);

        for event in listener.incoming() {
            let this = this.clone();
            pool.execute(move || handle(event.unwrap(), &this));
        }

        // We should never get Here
        unreachable!()
    }

    /// Add a new default header to the server.
    /// This will be added to every response if it is not already present.
    ///
    /// This will be added to every response
    /// ## Example
    /// ```rust
    /// # use afire::{Server, Header};
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080)
    ///     // Add a default header to the response
    ///     .default_header("X-Server", "afire");
    /// ```
    pub fn default_header(self, key: impl Into<HeaderType>, value: impl AsRef<str>) -> Self {
        let mut headers = self.default_headers;
        let header = Header::new(key, value);
        trace!("😀 Adding Server Header ({})", header);
        headers.push(header);

        Server {
            default_headers: headers,
            ..self
        }
    }

    /// Set the timeout for the socket.
    /// This will ensure that the server will not hang on a request for too long.
    /// By default there is no timeout.
    ///
    /// ## Example
    /// ```rust,no_run
    /// # use std::time::Duration;
    /// # use afire::Server;
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080)
    ///     // Set socket timeout
    ///     .socket_timeout(Duration::from_secs(5));
    /// ```
    pub fn socket_timeout(self, socket_timeout: Duration) -> Self {
        trace!("⏳ Setting Socket timeout to {:?}", socket_timeout);

        Server {
            socket_timeout: Some(socket_timeout),
            ..self
        }
    }

    /// Set the keep alive state of the server.
    /// This will determine if the server will keep the connection alive after a request.
    /// By default this is true.
    /// If you aren't using a threadpool, you may want to set this to false.
    /// ## Example
    /// ```rust
    /// # use afire::Server;
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080)
    ///     // Disable Keep Alive
    ///     .keep_alive(false);
    /// ```
    pub fn keep_alive(self, keep_alive: bool) -> Self {
        trace!("🔁 Setting Keep Alive to {}", keep_alive);

        Server { keep_alive, ..self }
    }

    /// Set the state of a server.
    /// The state will be available to stateful routes ([`Server::stateful_route`]) and middleware.
    /// It is not mutable, so you will need to use an atomic or sync type to mutate it.
    ///
    /// ## Example
    /// ```rust,no_run
    /// # use afire::{Server, Response, Method};
    /// # use std::sync::atomic::{AtomicU32, Ordering};
    /// // Create a server for localhost on port 8080
    /// // Note: We can omit the type parameter here because we are setting the state
    /// let mut server = Server::new("localhost", 8080)
    ///     // Set server wide state
    ///     .state(AtomicU32::new(0));
    ///
    /// // Add a stateful route to increment the state
    /// server.stateful_route(Method::GET, "/", |state, _req| {
    ///     Response::new().text(state.fetch_add(1, Ordering::Relaxed))
    /// });
    /// ```
    pub fn state(self, state: State) -> Self {
        trace!("📦️ Setting Server State [{}]", type_name::<State>());

        Self {
            state: Some(Arc::new(state)),
            ..self
        }
    }

    /// Set the panic handler, which is called if a route or middleware panics.
    /// This is only available if the `panic_handler` feature is enabled.
    /// If you don't set it, the default response is 500 "Internal Server Error :/".
    /// Be sure that your panic handler wont panic, because that will just panic the whole application.
    /// ## Example
    /// ```rust
    /// # use afire::{Server, Response, Status};
    /// # let mut server = Server::<()>::new("localhost", 8080);
    /// // Set the panic handler response
    /// server.error_handler(|_state, _req, err| {
    ///     Response::new()
    ///         .status(Status::InternalServerError)
    ///         .text(format!("Internal Server Error: {}", err))
    /// });
    /// ```
    pub fn error_handler(
        &mut self,
        res: impl Fn(Option<Arc<State>>, &Box<Result<Rc<Request>>>, String) -> Response
            + Send
            + Sync
            + 'static,
    ) {
        trace!("✌ Setting Error Handler");

        self.error_handler = Box::new(res);
    }

    /// Create a new route.
    /// The path can contain parameters, which are defined with `{...}`, as well as wildcards, which are defined with `*`.
    /// (`**` lets you math anything after the wildcard, including `/`)
    /// ## Example
    /// ```rust
    /// # use afire::{Server, Response, Header, Method, Content};
    /// # let mut server = Server::<()>::new("localhost", 8080);
    /// // Define a route
    /// server.route(Method::GET, "/greet/{name}", |req| {
    ///     let name = req.param("name").unwrap();
    ///
    ///     Response::new()
    ///         .text(format!("Hello, {}!", name))
    ///         .content(Content::TXT)
    /// });
    /// ```
    pub fn route(
        &mut self,
        method: Method,
        path: impl AsRef<str>,
        handler: impl Fn(&Request) -> Response + Send + Sync + 'static,
    ) -> &mut Self {
        let path = path.as_ref().to_owned();
        trace!("🚗 Adding Route {} {}", method, path);

        self.routes
            .push(Route::new(method, path, Box::new(handler)));
        self
    }

    /// Create a new stateful route.
    /// Is the same as [`Server::route`], but the state is passed as the first parameter.
    /// (See [`Server::state`])
    ///
    /// Note: If you add a stateful route, you must also set the state or starting the sever will return an error.
    /// ## Example
    /// ```rust
    /// # use afire::{Server, Response, Header, Method};
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<u32>::new("localhost", 8080)
    ///    .state(101);
    ///
    /// // Define a route
    /// server.stateful_route(Method::GET, "/nose", |sta, req| {
    ///     Response::new().text(sta.to_string())
    /// });
    /// ```
    pub fn stateful_route(
        &mut self,
        method: Method,
        path: impl AsRef<str>,
        handler: impl Fn(Arc<State>, &Request) -> Response + Send + Sync + 'static,
    ) -> &mut Self {
        let path = path.as_ref().to_owned();
        trace!("🚗 Adding Route {} {}", method, path);

        self.routes
            .push(Route::new_stateful(method, path, Box::new(handler)));
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
    pub fn app(&self) -> Arc<State> {
        self.state.as_ref().unwrap().clone()
    }

    fn check(&self) -> Result<()> {
        if self.state.is_none() && self.routes.iter().any(|x| x.is_stateful()) {
            return Err(StartupError::NoState.into());
        }

        if self.socket_timeout == Some(Duration::ZERO) {
            return Err(StartupError::InvalidSocketTimeout.into());
        }

        Ok(())
    }
}
