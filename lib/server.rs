// Import STD libraries
use std::fmt;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::str;
use std::sync::{Arc, RwLock};
use std::time::Duration;

// Feature Imports
#[cfg(feature = "panic_handler")]
use std::panic;

// Import local files
use crate::handle::{handle_connection, response_http};
use crate::header::Header;
use crate::method::Method;
use crate::middleware::Middleware;
use crate::request::Request;
use crate::response::Response;
use crate::route::Route;
use crate::thread_pool::ThreadPool;
use crate::VERSION;

/// Defines a server.
pub struct Server {
    /// Port to listen on.
    pub port: u16,

    /// Ip address to listen on.
    pub ip: Ipv4Addr,

    /// Default Buffer Size
    ///
    /// Needs to be big enough to hold a the request headers
    /// in order to read the content length (1024 seams to work)
    pub buff_size: usize,

    /// Routes to handle.
    pub routes: Vec<Route>,

    // Other stuff
    /// Middleware
    pub middleware: Vec<Box<dyn Middleware + Send + Sync>>,

    /// Default response for internal server errors
    #[cfg(feature = "panic_handler")]
    pub error_handler: Box<dyn Fn(Request, String) -> Response + Send + Sync>,

    /// Headers automatically added to every response.
    pub default_headers: Vec<Header>,

    /// Socket Timeout
    pub socket_timeout: Option<Duration>,

    /// Run server
    ///
    /// Really just for testing.
    pub run: bool,
}

unsafe impl Sync for Server {}

/// Implementations for Server
impl Server {
    /// Creates a new server.
    ///
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::Server;
    ///
    /// // Create a server for localhost on port 8080
    /// // Note: The server has not been started yet
    /// let mut server: Server = Server::new("localhost", 8080);
    /// ```
    pub fn new<T>(raw_ip: T, port: u16) -> Server
    where
        T: fmt::Display,
    {
        trace!("🐍 Initializing Server v{}", VERSION);

        let mut raw_ip = raw_ip.to_string();
        let mut ip: [u8; 4] = [0; 4];

        // If the ip is localhost, use the loop back ip
        if raw_ip == "localhost" {
            raw_ip = String::from("127.0.0.1");
        }

        // Parse the ip to an array
        let split_ip = raw_ip.split('.').collect::<Vec<&str>>();

        if split_ip.len() != 4 {
            panic!("Invalid Server IP");
        }
        for i in 0..4 {
            let octet = split_ip[i].parse::<u8>().expect("Invalid Server IP");
            ip[i] = octet;
        }

        let ip = Ipv4Addr::from(ip);

        Server {
            port,
            ip,
            buff_size: 1024,
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
    /// let mut server: Server = Server::new("localhost", 8080);
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
    pub fn start(&self) -> Option<()> {
        // Exit if the server should not run
        if !self.run {
            return Some(());
        }

        trace!("✨ Starting Server [{}:{}]", self.ip, self.port);

        let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(self.ip), self.port)).ok()?;

        for event in listener.incoming() {
            // Read stream into buffer
            let mut stream = event.ok()?;

            // Get the response from the handler
            // Uses the most recently defined route that matches the request
            let (req, res) = handle_connection(&stream, self);

            if res.close {
                continue;
            }

            let end_res = res.clone();
            let end_req = req.clone();
            let response = response_http(self, req, res);

            // Send the response
            let _ = stream.write_all(&response);
            stream.flush().ok()?;

            // Run end middleware
            for middleware in self.middleware.iter().rev() {
                #[cfg(feature = "panic_handler")]
                let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    middleware.end(end_req.clone(), end_res.clone())
                }));

                #[cfg(not(feature = "panic_handler"))]
                middleware.end(end_req.clone(), end_res.clone());
            }
        }

        // We should Never Get Here
        None
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
    /// let mut server: Server = Server::new("localhost", 8080);
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
    pub fn start_threaded(self, threads: usize) -> Option<()> {
        // Exit if the server should not run
        if !self.run {
            return Some(());
        }

        trace!(
            "✨ Starting Server [{}:{}] ({} threads)",
            self.ip,
            self.port,
            threads
        );

        let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(self.ip), self.port)).ok()?;

        let pool = ThreadPool::new(threads);
        let this = Arc::new(RwLock::new(self));

        for event in listener.incoming() {
            let this = Arc::clone(&this);
            pool.execute(move || {
                let this = this.read().unwrap();

                // Read stream into buffer
                let mut stream = event.unwrap();

                // Get the response from the handler
                // Uses the most recently defined route that matches the request
                let (req, res) = handle_connection(&stream, &this);

                if res.close {
                    return;
                }

                let end_res = res.clone();
                let end_req = req.clone();
                let response = response_http(&this, req, res);

                let _ = stream.write_all(&response);
                stream.flush().unwrap();

                // Run end middleware
                for middleware in this.middleware.iter().rev() {
                    #[cfg(feature = "panic_handler")]
                    let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                        middleware.end(end_req.clone(), end_res.clone())
                    }));

                    #[cfg(not(feature = "panic_handler"))]
                    middleware.end(end_req.clone(), end_res.clone());
                }
            });
        }

        unreachable!()
    }

    /// Set the satrting buffer size. The default is `1024`
    ///
    /// Needs to be big enough to hold a the request headers
    /// in order to read the content length (1024 seams to work)
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::Server;
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server: Server = Server::new("localhost", 8080)
    ///     .buffer(2048);
    /// ```
    pub fn buffer(self, buf: usize) -> Server {
        trace!("🥫 Setting Buffer to {} bytes", buf);

        Server {
            buff_size: buf,
            ..self
        }
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
    /// let mut server: Server = Server::new("localhost", 8080)
    ///     // Add a default header to the response
    ///     .default_header("Content-Type", "text/plain");
    ///
    /// // Start the server
    /// // As always, this is blocking
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn default_header<T, K>(self, key: T, value: K) -> Server
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
    /// let mut server: Server = Server::new("localhost", 8080)
    ///     // Set socket timeout
    ///     .socket_timeout(Duration::from_secs(1));
    ///
    /// // Start the server
    /// // As always, this is blocking
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn socket_timeout(self, socket_timeout: Duration) -> Server {
        trace!("⏳ Setting Socket timeout to {:?}", socket_timeout);

        Server {
            socket_timeout: Some(socket_timeout),
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
    /// let mut server: Server = Server::new("localhost", 8080);
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
    /// let mut server: Server = Server::new("localhost", 8080);
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
        res: impl Fn(Request, String) -> Response + Send + Sync + 'static,
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
    /// let mut server: Server = Server::new("localhost", 8080);
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
        handler: impl Fn(Request) -> Response + Send + Sync + 'static,
    ) where
        T: AsRef<str>,
    {
        let path = path.as_ref().to_owned();
        trace!("🚗 Adding Route {} {}", method, path);

        self.routes
            .push(Route::new(method, path, Box::new(handler)));
    }
}
