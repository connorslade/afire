// Import STD libraries
use std::cell::RefCell;
use std::fmt;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::str;
use std::time::Duration;

// Feature Imports
#[cfg(feature = "panic_handler")]
use std::panic;

// Import local files
use crate::common::reason_phrase;
use crate::handle::handle_connection;
use crate::header::{headers_to_string, Header};
use crate::method::Method;
use crate::middleware::{MiddleResponse, Middleware};
use crate::request::Request;
use crate::response::Response;
use crate::route::Route;
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
    pub middleware: Vec<Box<RefCell<dyn Middleware>>>,

    /// Default response for internal server errors
    #[cfg(feature = "panic_handler")]
    pub error_handler: Box<dyn Fn(Request, String) -> Response>,

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
                    .header(Header::new("Content-Type", "text/plain"))
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
    ///         .header(Header::new("Content-Type", "text/plain"))
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

        let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(self.ip), self.port)).ok()?;

        for event in listener.incoming() {
            // Read stream into buffer
            let mut stream = event.ok()?;

            if self.socket_timeout.is_some() {
                stream.set_read_timeout(self.socket_timeout).unwrap();
                stream.set_write_timeout(self.socket_timeout).unwrap();
            }

            // Get the response from the handler
            // Uses the most recently defined route that matches the request
            let (req, mut res) = handle_connection(
                &stream,
                &self.middleware,
                #[cfg(feature = "panic_handler")]
                &self.error_handler,
                &self.routes,
                self.buff_size,
            );

            for middleware in &mut self.middleware.iter().rev() {
                match middleware.borrow_mut().post(req.clone(), res.clone()) {
                    MiddleResponse::Continue => {}
                    MiddleResponse::Add(i) => res = i,
                    MiddleResponse::Send(i) => {
                        res = i;
                        break;
                    }
                }
            }

            if res.close {
                continue;
            }

            // Add default headers to response
            let mut headers = res.headers;
            headers.append(&mut self.default_headers.clone());

            // Add content-length header to response
            headers.push(Header::new("Content-Length", &res.data.len().to_string()));

            // Convert the response to a string
            // TODO: Use Bytes instead of String
            let mut response = format!(
                "HTTP/1.1 {} {}\r\n{}\r\n\r\n",
                res.status,
                match res.reason {
                    Some(i) => i,
                    None => reason_phrase(res.status),
                },
                headers_to_string(headers)
            )
            .as_bytes()
            .to_vec();

            // Add Bytes of data to response
            response.append(&mut res.data);

            // Send the response
            let _ = stream.write_all(&response);
            stream.flush().ok()?;
        }

        // We should Never Get Here
        None
    }

    /// Get the ip a server is listening on as a string
    ///
    /// For example, "127.0.0.1"
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::Server;
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server: Server = Server::new("localhost", 8080);
    ///
    /// // Get the ip a server is listening on as a string
    /// assert_eq!("127.0.0.1", server.ip_string());
    /// ```
    #[deprecated(since = "0.3.0", note = "Instead use .ip.to_string()")]
    pub fn ip_string(&self) -> String {
        self.ip
            .octets()
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(".")
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
    ///     .default_header(Header::new("Content-Type", "text/plain"));
    ///
    /// // Start the server
    /// // As always, this is blocking
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn default_header(self, header: Header) -> Server {
        let mut headers = self.default_headers;
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
    pub fn error_handler(&mut self, res: fn(Request, String) -> Response) {
        self.error_handler = Box::new(res);
    }

    /// Define the panic handler but with closures!
    ///
    /// Basicity just [`Server::error_handler`] but with closures
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
    /// server.error_handler_c(Box::new(|_req, err| {
    ///     Response::new()
    ///         .status(500)
    ///         .text(format!("Internal Server Error: {}", err))
    /// }));
    ///
    /// // Start the server
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    #[cfg(feature = "panic_handler")]
    pub fn error_handler_c(&mut self, res: Box<dyn Fn(Request, String) -> Response>) {
        self.error_handler = res;
    }

    /// Create a new route the runs for all methods and paths
    ///
    /// May be useful for a 404 page as the most recently defined route takes priority
    /// so by defining this route first it would trigger if nothing else matches
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Server, Response, Header, Method};
    ///
    /// // Starts a server for localhost on port 8080
    /// let mut server: Server = Server::new("localhost", 8080);
    ///
    /// // Define 404 page
    /// // Because this is defined first, it will take a low priority
    /// server.all(|req| {
    ///     Response::new()
    ///         .status(404)
    ///         .text("The page you are looking for does not exist :/")
    ///         .header(Header::new("Content-Type", "text/plain"))
    /// });
    ///
    /// // Define a route
    /// // As this is defined last, it will take a high priority
    /// server.route(Method::GET, "/nose", |req| {
    ///     Response::new()
    ///         .status(200)
    ///         .text("N O S E")
    ///         .header(Header::new("Content-Type", "text/plain"))
    /// });
    ///
    /// // Starts the server
    /// // This is blocking
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    #[deprecated(since = "0.3.0", note = "Instead use .route(Method::ANY, \"*\", ...)")]
    pub fn all(&mut self, handler: fn(Request) -> Response) {
        self.routes
            .push(Route::new(Method::ANY, "**".to_owned(), Box::new(handler)));
    }

    /// Create a new route the runs for all methods and paths
    ///
    /// May be useful for a 404 page as the most recently defined route takes priority
    /// so by defining this route first it would trigger if nothing else matches
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Server, Response, Header, Method};
    ///
    /// // Starts a server for localhost on port 8080
    /// let mut server: Server = Server::new("localhost", 8080);
    ///
    /// // Define 404 page
    /// // Because this is defined first, it will take a low priority
    /// server.all_c(Box::new(|req| {
    ///     Response::new()
    ///         .status(404)
    ///         .text("The page you are looking for does not exist :/")
    ///         .header(Header::new("Content-Type", "text/plain"))
    /// }));
    ///
    /// // Define a route
    /// // As this is defined last, it will take a high priority
    /// server.route(Method::GET, "/nose", |req| {
    ///     Response::new()
    ///         .status(200)
    ///         .text("N O S E")
    ///         .header(Header::new("Content-Type", "text/plain"))
    /// });
    ///
    /// // Starts the server
    /// // This is blocking
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    #[deprecated(since = "0.3.0", note = "Instead use .route(Method::ANY, \"*\", ...)")]
    pub fn all_c(&mut self, handler: Box<dyn Fn(Request) -> Response>) {
        self.routes
            .push(Route::new(Method::ANY, "**".to_owned(), handler));
    }

    /// Create a new route for any type of request
    ///
    /// Basicity just [`Server::any`] but with closures
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Server, Response, Header};
    ///
    /// // Starts a server for localhost on port 8080
    /// let mut server: Server = Server::new("localhost", 8080);
    ///
    /// // Define a route
    /// server.any("/nose", |req| {
    ///     Response::new()
    ///         .status(200)
    ///         .text("N O S E")
    ///         .header(Header::new("Content-Type", "text/plain"))
    /// });
    ///
    /// // Starts the server
    /// // This is blocking
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    /// Now you can make any type of request to `/nose` and it will return a 200
    #[deprecated(since = "0.1.5", note = "Instead use .route(Method::ANY...)")]
    pub fn any<T>(&mut self, path: T, handler: fn(Request) -> Response)
    where
        T: fmt::Display,
    {
        self.routes
            .push(Route::new(Method::ANY, path.to_string(), Box::new(handler)));
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
    ///         .header(Header::new("Content-Type", "text/plain"))
    /// });
    ///
    /// // Starts the server
    /// // This is blocking
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn route<T>(&mut self, method: Method, path: T, handler: fn(Request) -> Response)
    where
        T: fmt::Display,
    {
        self.routes
            .push(Route::new(method, path.to_string(), Box::new(handler)));
    }

    /// Define a new route with a closure as a handler
    ///
    /// Basicity just [`Server::route`] but with closures
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Server, Response, Header, Method};
    ///
    /// use std::cell::RefCell;
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server: Server = Server::new("localhost", 8080);
    ///
    /// let cell = RefCell::new(0);
    ///
    /// // Define a route with a closure
    /// server.route_c(Method::GET, "/nose", Box::new(move |req| {
    ///     cell.replace_with(|&mut old| old + 1);
    ///
    ///     Response::new()
    ///         .status(200)
    ///         .text("N O S E")
    ///         .header(Header::new("Content-Type", "text/plain"))
    /// }));
    ///
    /// // Starts the server
    /// // This is blocking
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn route_c<T>(&mut self, method: Method, path: T, handler: Box<dyn Fn(Request) -> Response>)
    where
        T: fmt::Display,
    {
        self.routes
            .push(Route::new(method, path.to_string(), handler));
    }
}
