// Import STD libraries
use std::any::type_name;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::str;
use std::sync::Arc;
use std::time::Duration;

// Import local files
use crate::{
    error::Result, error::StartupError, handle::handle, internal::common, thread_pool::ThreadPool,
    Header, Method, Middleware, Request, Response, Route, VERSION,
};

/// Defines a server.
pub struct Server<State = ()>
where
    State: 'static + Send + Sync,
{
    /// Port to listen on.
    pub port: u16,

    /// Ip address to listen on.
    pub ip: Ipv4Addr,

    /// Routes to handle.
    pub routes: Vec<Route<State>>,

    // Other stuff
    /// Middleware
    pub middleware: Vec<Box<dyn Middleware + Send + Sync>>,

    /// Server wide App State
    pub state: Option<Arc<State>>,

    /// Default response for internal server errors
    #[cfg(feature = "panic_handler")]
    pub error_handler: Box<dyn Fn(Result<Request>, String) -> Response + Send + Sync>,

    /// Headers automatically added to every response.
    pub default_headers: Vec<Header>,

    /// Socket Timeout
    pub socket_timeout: Option<Duration>,

    /// Run server
    ///
    /// Really just for testing.
    pub run: bool,
}

/// Implementations for Server
impl<State> Server<State>
where
    State: Send + Sync,
{
    /// Creates a new server.
    ///
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::Server;
    ///
    /// // Create a server for localhost on port 8080
    /// // Note: The server has not been started yet
    /// let mut server = Server::<()>::new("localhost", 8080);
    /// ```
    pub fn new<T>(raw_ip: T, port: u16) -> Self
    where
        T: AsRef<str>,
    {
        trace!("🐍 Initializing Server v{}", VERSION);

        let ip = common::parse_ip(raw_ip.as_ref()).unwrap();

        Server {
            port,
            ip: Ipv4Addr::from(ip),
            routes: Vec::new(),
            middleware: Vec::new(),
            run: true,

            #[cfg(feature = "panic_handler")]
            error_handler: Box::new(|_, err| {
                Response::new()
                    .status(500)
                    .text(format!("Internal Server Error :/\nError: {}", err))
                    .header("Content-Type", "text/plain")
            }),

            default_headers: vec![Header::new("Server", format!("afire/{}", VERSION))],
            socket_timeout: None,
            state: None,
        }
    }

    /// Start the server.
    ///
    /// Will be blocking.
    ///
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Server, Response, Header, Method};
    ///
    /// // Starts a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080);
    ///
    /// // Define a route
    /// server.route(Method::GET, "/", |req| {
    ///     Response::new()
    ///         .status(200)
    ///         .text("N O S E")
    ///         .header("Content-Type", "text/plain")
    /// });
    ///
    /// // Starts the server
    /// // This is blocking
    /// # // Keep the server from starting and blocking the main thread
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn start(&self) -> Result<()> {
        // Exit if the server should not run
        if !self.run {
            return Ok(());
        }

        self.check()?;

        trace!("✨ Starting Server [{}:{}]", self.ip, self.port);

        let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(self.ip), self.port))?;

        for event in listener.incoming() {
            handle(&mut event?, self);
        }

        // We should Never Get Here
        unreachable!()
    }

    /// Start the server with a threadpool.
    ///
    /// Will be blocking.
    ///
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Server, Response, Header, Method};
    ///
    /// // Starts a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080);
    ///
    /// // Define a route
    /// server.route(Method::GET, "/", |req| {
    ///     Response::new()
    ///         .status(200)
    ///         .text("N O S E")
    ///         .header("Content-Type", "text/plain")
    /// });
    ///
    /// // Starts the server
    /// // This is blocking
    /// # // Keep the server from starting and blocking the main thread
    /// # server.set_run(false);
    /// server.start_threaded(4).unwrap();
    /// ```
    pub fn start_threaded(self, threads: usize) -> Result<()> {
        // Exit if the server should not run
        if !self.run {
            return Ok(());
        }

        self.check()?;

        trace!(
            "✨ Starting Server [{}:{}] ({} threads)",
            self.ip,
            self.port,
            threads
        );

        let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(self.ip), self.port))?;

        let pool = ThreadPool::new(threads);
        let this = Arc::new(self);

        for event in listener.incoming() {
            let this = this.clone();
            pool.execute(move || handle(&mut event.unwrap(), &this));
        }

        unreachable!()
    }

    /// Add a new default header to the response
    ///
    /// This will be added to every response
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Server, Header};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080)
    ///     // Add a default header to the response
    ///     .default_header("Content-Type", "text/plain");
    ///
    /// // Start the server
    /// // As always, this is blocking
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn default_header<T, K>(self, key: T, value: K) -> Self
    where
        T: AsRef<str>,
        K: AsRef<str>,
    {
        let mut headers = self.default_headers;
        let header = Header::new(key.as_ref(), value.as_ref());
        trace!("😀 Adding Server Header ({})", header);
        headers.push(header);

        Server {
            default_headers: headers,
            ..self
        }
    }

    /// Set the socket Read / Write Timeout
    ///
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use std::time::Duration;
    /// use afire::Server;
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080)
    ///     // Set socket timeout
    ///     .socket_timeout(Duration::from_secs(1));
    ///
    /// // Start the server
    /// // As always, this is blocking
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn socket_timeout(self, socket_timeout: Duration) -> Self {
        trace!("⏳ Setting Socket timeout to {:?}", socket_timeout);

        Server {
            socket_timeout: Some(socket_timeout),
            ..self
        }
    }

    /// Set the state of a server
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::Server;
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<u32>::new("localhost", 8080)
    ///     // Set server wide state
    ///     .state(101);
    ///
    /// // Start the server
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn state(self, state: State) -> Self {
        trace!("📦️ Setting Server State [{}]", type_name::<State>());

        Self {
            state: Some(Arc::new(state)),
            ..self
        }
    }

    /// Keep a server from starting
    ///
    /// Only used for testing
    ///
    /// It would be a really dumb idea to use this
    ///
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::Server;
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080);
    ///
    /// // Keep the server from starting and blocking the main thread
    /// server.set_run(false);
    ///
    /// // 'Start' the server
    /// server.start().unwrap();
    /// ```
    // I want to change this to be Server builder style
    // But that will require modifying *every* example so that can wait...
    #[doc(hidden)]
    pub fn set_run(&mut self, run: bool) {
        trace!("👟 {} Server", if run { "Enableing" } else { "Disableing" });

        self.run = run;
    }

    /// Set the panic handler response
    ///
    /// Default response is 500 "Internal Server Error :/"
    ///
    /// This is only available if the `panic_handler` feature is enabled
    ///
    /// Make sure that this wont panic because then the thread will crash
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Server, Response};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080);
    ///
    /// // Set the panic handler response
    /// server.error_handler(|_req, err| {
    ///     Response::new()
    ///         .status(500)
    ///         .text(format!("Internal Server Error: {}", err))
    /// });
    ///
    /// // Start the server
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    #[cfg(feature = "panic_handler")]
    pub fn error_handler(
        &mut self,
        res: impl Fn(Result<Request>, String) -> Response + Send + Sync + 'static,
    ) {
        trace!("✌ Setting Error Handler");

        self.error_handler = Box::new(res);
    }

    /// Create a new route for specified requests
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Server, Response, Header, Method};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080);
    ///
    /// // Define a route
    /// server.route(Method::GET, "/nose", |req| {
    ///     Response::new()
    ///         .status(200)
    ///         .text("N O S E")
    ///         .header("Content-Type", "text/plain")
    /// });
    ///
    /// // Starts the server
    /// // This is blocking
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn route<T>(
        &mut self,
        method: Method,
        path: T,
        handler: impl Fn(&Request) -> Response + Send + Sync + 'static,
    ) where
        T: AsRef<str>,
    {
        let path = path.as_ref().to_owned();
        trace!("🚗 Adding Route {} {}", method, path);

        self.routes
            .push(Route::new(method, path, Box::new(handler)));
    }

    /// Create a new stateful route
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Server, Response, Header, Method};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<u32>::new("localhost", 8080)
    ///    .state(101);
    ///
    /// // Define a route
    /// server.stateful_route(Method::GET, "/nose", |sta, req| {
    ///     Response::new().text(sta.to_string())
    /// });
    ///
    /// // Starts the server
    /// // This is blocking
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn stateful_route<T>(
        &mut self,
        method: Method,
        path: T,
        handler: impl Fn(Arc<State>, &Request) -> Response + Send + Sync + 'static,
    ) where
        T: AsRef<str>,
    {
        let path = path.as_ref().to_owned();
        trace!("🚗 Adding Route {} {}", method, path);

        self.routes
            .push(Route::new_stateful(method, path, Box::new(handler)));
    }

    fn check(&self) -> Result<()> {
        if self.state.is_none() && self.routes.iter().any(|x| x.is_stateful()) {
            return Err(StartupError::NoState.into());
        }

        Ok(())
    }
}
