// Import STD libraries
use std::fmt;
use std::io;
use std::io::Read;
use std::io::Write;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::str;
use std::time::Duration;

// Feature Imports
#[cfg(feature = "panic_handler")]
use std::panic;

// #[cfg(feature = "thread_pool")]
// TODO: Add Back Threadpool
// use super::threadpool::ThreadPool;

// Import local files
use super::common::reason_phrase;
use super::header::{headers_to_string, Header};
use super::http;
use super::method::Method;
use super::request::Request;
use super::response::Response;
use super::route::Route;
use super::VERSION;

/// Default Buffer Size
///
/// Needs to be big enough to hold a the request headers
/// in order to read the content length (1024 seams to work)
const BUFF_SIZE: usize = 1024;

/// Defines a server.
pub struct Server {
    /// Port to listen on.
    pub port: u16,

    /// Ip address to listen on.
    pub ip: [u8; 4],

    /// Routes to handle.
    pub routes: Vec<Route>,

    // Other stuff
    /// Middleware
    pub middleware: Vec<Box<dyn Fn(&Request) -> Option<Response>>>,

    /// Default response for internal server errors
    #[cfg(feature = "panic_handler")]
    pub error_handler: Box<dyn Fn(Request, String) -> Response>,

    /// Headers automatically added to every response.
    pub default_headers: Option<Vec<Header>>,

    /// Socket Timeout
    pub socket_timeout: Option<Duration>,

    /// Run server
    ///
    /// Really just for testing.
    run: bool,
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

        Server {
            port,
            ip,
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

            default_headers: Some(vec![Header::new("Server", format!("afire/{}", VERSION))]),
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

        let listener = init_listener(self.ip, self.port).ok()?;

        for event in listener.incoming() {
            // Read stream into buffer
            let mut stream = event.ok()?;
            stream.set_read_timeout(self.socket_timeout).unwrap();
            stream.set_write_timeout(self.socket_timeout).unwrap();

            // Get the response from the handler
            // Uses the most recently defined route that matches the request
            let mut res =
                handle_connection(&stream, &self.middleware, &self.error_handler, &self.routes);

            // Add default headers to response
            let mut headers = res.headers;
            headers.append(&mut self.default_headers.clone().unwrap_or_default());

            // Add content-length header to response
            headers.push(Header::new("Content-Length", &res.data.len().to_string()));

            // Convert the response to a string
            // TODO: Use Bytes instead of String
            let mut response = format!(
                "HTTP/1.1 {} {}\r\n{}\r\n\r\n",
                res.status,
                reason_phrase(res.status),
                headers_to_string(headers)
            )
            .as_bytes()
            .to_vec();

            // Add Bytes of data to response
            response.append(&mut res.data);

            // Send the response
            let _ = stream.write_all(&response);
            stream.flush().unwrap();
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
    pub fn ip_string(&self) -> String {
        let ip = self.ip;
        format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
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
    /// let mut server: Server = Server::new("localhost", 8080);
    ///
    /// // Add a default header to the response
    /// server.add_default_header(Header::new("Content-Type", "text/plain"));
    ///
    /// // Start the server
    /// // As always, this is blocking
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn add_default_header(&mut self, header: Header) {
        self.default_headers
            .as_mut()
            .unwrap_or(&mut Vec::<Header>::new())
            .push(header);
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
    /// let mut server: Server = Server::new("localhost", 8080);
    ///
    /// // Set socket timeout
    /// server.socket_timeout(Some(Duration::from_secs(1)));
    ///
    /// // Start the server
    /// // As always, this is blocking
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn socket_timeout(&mut self, socket_timeout: Option<Duration>) {
        self.socket_timeout = socket_timeout;
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
    pub fn set_run(&mut self, run: bool) {
        self.run = run;
    }

    /// Add a new middleware to the server
    ///
    /// Will be executed before any routes are handled
    ///
    /// You will have access to the request object
    /// You can send a response but it will keep normal routes from being handled
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Server};
    ///
    /// // Starts a server for localhost on port 8080
    /// let mut server: Server = Server::new("localhost", 8080);
    ///
    /// // Add some middleware
    /// server.middleware(Box::new(|req| {
    ///     // Do something with the request
    ///     // Return a `None` to continue to the next middleware / route
    ///     // Return a `Some` to send a response
    ///    None
    ///}));
    ///
    /// // Starts the server
    /// // This is blocking
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn middleware(&mut self, handler: Box<dyn Fn(&Request) -> Option<Response>>) {
        self.middleware.push(handler);
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
    /// Basicly just [`Server::error_handler`] but with closures
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
    pub fn all(&mut self, handler: fn(Request) -> Response) {
        self.routes
            .push(Route::new(Method::ANY, "*".to_string(), Box::new(handler)));
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
    pub fn all_c(&mut self, handler: Box<dyn Fn(Request) -> Response>) {
        self.routes
            .push(Route::new(Method::ANY, "*".to_string(), handler));
    }

    /// Create a new route for any type of request
    ///
    /// Basicly just [`Server::any`] but with closures
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
    /// Basicly just [`Server::route`] but with closures
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

/// Handle a request
fn handle_connection(
    mut stream: &TcpStream,
    middleware: &[Box<dyn Fn(&Request) -> Option<Response>>],
    error_handler: &Box<dyn Fn(Request, String) -> Response>,
    routes: &[Route],
) -> Response {
    // Init (first) Buffer
    let mut buffer = vec![0; BUFF_SIZE];

    // Read stream into buffer
    match stream.read(&mut buffer) {
        Ok(_) => {}
        Err(_) => return quick_err("Error Reading Stream", 500),
    };

    // Get buffer as string
    let buffer_clone = buffer.clone();
    let stream_string = match str::from_utf8(&buffer_clone) {
        Ok(s) => s,
        Err(_) => return quick_err("Currently no support for non utf-8 characters...", 500),
    };

    // Get Content-Length header
    // If header shows thar more space is needed,
    // make a new buffer read the rest of the stream and add it to the first buffer
    // This could cause a proformance hit but is actually seams to be fasy enough
    #[cfg(feature = "dynamic_resize")]
    for i in http::get_request_headers(stream_string.to_string()) {
        if i.name != "Content-Length" {
            continue;
        }
        let header_size = http::get_header_size(stream_string.to_string());
        let content_length = i.value.parse::<usize>().unwrap_or(0);
        let new_buffer_size = content_length as i64 + header_size as i64 - BUFF_SIZE as i64;
        if new_buffer_size > 0 {
            let mut new_buffer = vec![0; new_buffer_size as usize];
            match stream.read(&mut new_buffer) {
                Ok(_) => {}
                Err(_) => return quick_err("Error Reading Stream", 500),
            };
            buffer.append(&mut new_buffer);
        }
        break;
    }

    // TODO: Make this work with non utf8 stuff too
    let stream_string = match str::from_utf8(&buffer) {
        Ok(i) => i.trim_matches(char::from(0)),
        Err(_) => return quick_err("No support for non utf-8 chars\nFor now", 500),
    };

    // Make Request Object
    let req_method = http::get_request_method(stream_string.to_string());
    let req_path = http::get_request_path(stream_string.to_string());
    let req_query = http::get_request_query(stream_string.to_string());
    let body = http::get_request_body(stream_string.to_string());
    let headers = http::get_request_headers(stream_string.to_string());
    #[cfg(feature = "cookies")]
    let cookies = http::get_request_cookies(stream_string.to_string());
    let req = Request::new(
        req_method,
        &req_path,
        req_query,
        headers,
        #[cfg(feature = "cookies")]
        cookies,
        body,
        stream.peer_addr().unwrap().to_string(),
        stream_string.to_string(),
    );

    // Use middleware to handle request
    // If middleware returns a `None`, the request will be handled by earlier middleware then the routes
    for middleware in middleware.iter().rev() {
        match (middleware)(&req) {
            None => (),
            Some(res) => return res,
        }
    }

    // Loop through all routes and check if the request matches
    for route in routes.iter().rev() {
        if (req.method == route.method || route.method == Method::ANY)
            && (req.path == route.path || route.path == "*")
        {
            // Optionally enable automatic panic handling
            #[cfg(feature = "panic_handler")]
            {
                // let handler = .clone();
                let result =
                    panic::catch_unwind(panic::AssertUnwindSafe(|| (route.handler)(req.clone())));
                let err = match result {
                    Ok(i) => return i,
                    Err(e) => match e.downcast_ref::<&str>() {
                        Some(err) => err,
                        None => "",
                    },
                };
                return (error_handler)(req, err.to_string());
            }

            #[cfg(not(feature = "panic_handler"))]
            {
                return (route.handler)(req);
            }
        }
    }

    // If no route was found, return a default 404
    Response::new()
        .status(404)
        .text(format!("Cannot {} {}", req.method, req.path))
        .header(Header::new("Content-Type", "text/plain"))
}

/// Init Listaner
fn init_listener(ip: [u8; 4], port: u16) -> Result<TcpListener, io::Error> {
    TcpListener::bind(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3])),
        port,
    ))
}

/// Quick function to get a basic error response
fn quick_err(text: &str, code: u16) -> Response {
    Response::new()
        .status(code)
        .text(text)
        .header(Header::new("Content-Type", "text/plain"))
}
