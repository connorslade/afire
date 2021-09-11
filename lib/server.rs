// Import STD libraries
use std::io::Read;
use std::io::Write;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::str;

#[cfg(feature = "panic_handler")]
use std::panic;

// Import local files

use super::cookie::Cookie;
use super::header::{headers_to_string, Header};
use super::method::Method;
use super::query::Query;
use super::request::Request;
use super::response::Response;
use super::route::Route;

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

    /// Run server
    ///
    /// Really just for testing.
    run: bool,

    /// Default response for internal server errors
    #[cfg(feature = "panic_handler")]
    error_handler: fn(Request) -> Response,
    /// Headers automatically added to every response.
    default_headers: Option<Vec<Header>>,
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
    pub fn new(mut raw_ip: &str, port: u16) -> Server {
        let mut ip: [u8; 4] = [0; 4];

        // If the ip is localhost, use the loop back ip
        if raw_ip == "localhost" {
            raw_ip = "127.0.0.1";
        }

        // Parse the ip to an array
        let split_ip: Vec<&str> = raw_ip.split('.').collect();

        if split_ip.len() != 4 {
            panic!("Invalid Server IP");
        }
        for i in 0..4 {
            let octet: u8 = split_ip[i].parse::<u8>().expect("Invalid Server IP");
            ip[i] = octet;
        }

        Server {
            port,
            ip,
            routes: Vec::new(),
            middleware: Vec::new(),
            run: true,
            #[cfg(feature = "panic_handler")]
            error_handler: |_| {
                Response::new(
                    500,
                    "Internal Server Error :/",
                    vec![Header::new("Content-Type", "text/plain")],
                )
            },
            default_headers: Some(vec![Header::new("Server", "afire")]),
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
    ///     Response::new(
    ///         200,
    ///         "N O S E",
    ///         vec![Header::new("Content-Type", "text/plain")],
    ///     )
    /// });
    ///
    /// // Starts the server
    /// // This is blocking
    /// # // Keep the server from starting and blocking the main thread
    /// # server.set_run(false);
    /// server.start();
    /// ```
    pub fn start(&self) {
        // Exit if the server should not run
        if !self.run {
            return;
        }

        let listener = TcpListener::bind(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(
                self.ip[0], self.ip[1], self.ip[2], self.ip[3],
            )),
            self.port,
        ))
        .unwrap();

        for event in listener.incoming() {
            // Read stream into buffer
            let mut stream = event.unwrap();

            // Get the response from the handler
            // Uses the most recently defined route that matches the request
            let mut res = self.handle_connection(&stream);

            // Add default headers to response
            let mut headers = res.headers;
            headers.append(&mut self.default_headers.clone().unwrap_or_default());

            // Add content-length header to response
            headers.push(Header::new("Content-Length", &res.data.len().to_string()));

            // Convert the response to a string
            let mut response = format!(
                "HTTP/1.1 {} OK\r\n{}\r\n\r\n",
                res.status,
                headers_to_string(headers)
            )
            .as_bytes()
            .to_vec();

            // Add Bytes of data to response
            response.append(&mut res.data);

            // Send the response
            stream.write_all(&response).unwrap();
            stream.flush().unwrap();
        }
    }

    /// Handel a connection to the server
    // TODO: Try just expanding buffer if full and not relinging on content length
    fn handle_connection(&self, mut stream: &TcpStream) -> Response {
        // Init (first) Buffer
        let mut buffer = vec![0; BUFF_SIZE];

        // Read stream into buffer
        let _ = stream.read(&mut buffer).unwrap();

        // Get buffer as string
        let buffer_clone = buffer.clone();
        let stream_string = str::from_utf8(&buffer_clone).expect("Error parsing buffer data");

        // Get Content-Length header
        // If header shows thar more space is needed,
        // make a new buffer read the rest of the stream and add it to the first buffer
        for i in get_request_headers(stream_string.to_string()) {
            if i.name != "Content-Length" {
                continue;
            }
            let header_size = get_header_size(stream_string.to_string());
            let content_length = i.value.parse::<usize>().unwrap_or(0);
            let new_buffer_size = content_length as i64 + header_size as i64 - BUFF_SIZE as i64;
            if new_buffer_size > 0 {
                let mut new_buffer = vec![0; new_buffer_size as usize];
                let _ = stream.read(&mut new_buffer).unwrap();
                buffer.append(&mut new_buffer);
            }
            break;
        }

        let stream_string = str::from_utf8(&buffer).expect("Error parsing buffer data");

        // Make Request Object
        let req_method = get_request_method(stream_string.to_string());
        let req_path = get_request_path(stream_string.to_string());
        let req_query = get_request_query(stream_string.to_string());
        let body = get_request_body(stream_string.to_string());
        let headers = get_request_headers(stream_string.to_string());
        let cookies = get_request_cookies(stream_string.to_string());
        let req = Request::new(
            req_method,
            &req_path,
            req_query,
            headers,
            cookies,
            body,
            stream.peer_addr().unwrap().to_string(),
            stream_string.to_string(),
        );

        // Use middleware to handle request
        // If middleware returns a `None`, the request will be handled by earlier middleware then the routes
        for middleware in self.middleware.iter().rev() {
            match (middleware)(&req) {
                None => (),
                Some(res) => return res,
            }
        }

        // Loop through all routes and check if the request matches
        for route in self.routes.iter().rev() {
            if (req.method == route.method || route.method == Method::ANY)
                && (req.path == route.path || route.path == "*")
            {
                // Optionally enable automatic panic handling
                #[cfg(feature = "panic_handler")]
                {
                    let result = panic::catch_unwind(|| (route.handler)(req.clone()));
                    return result.ok().unwrap_or_else(|| (self.error_handler)(req));
                }

                #[cfg(not(feature = "panic_handler"))]
                {
                    return (route.handler)(req);
                }
            }
        }

        // If no route was found, return a default 404
        Response::new(
            404,
            "Not Found",
            vec![Header::new("Content-Type", "text/plain")],
        )
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
    /// server.start();
    /// ```
    pub fn set_run(&mut self, run: bool) {
        self.run = run;
    }

    #[cfg(feature = "panic_handler")]
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
    /// server.set_error_handler(|_req| {
    ///    Response::new(500, "Internal Server Error", vec![])
    /// });
    ///
    /// // Start the server
    /// # server.set_run(false);
    /// server.start();
    /// ```
    pub fn set_error_handler(&mut self, res: fn(Request) -> Response) {
        self.error_handler = res;
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
    /// server.start();
    /// ```
    pub fn add_default_header(&mut self, header: Header) {
        self.default_headers
            .as_mut()
            .unwrap_or(&mut Vec::<Header>::new())
            .push(header);
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
    ///     Response::new(
    ///         404,
    ///         "The page you are looking for does not exist :/",
    ///         vec![Header::new("Content-Type", "text/plain")],
    ///     )
    /// });
    ///
    /// // Define a route
    /// // As this is defined last, it will take a high priority
    /// server.route(Method::GET, "/nose", |req| {
    ///     Response::new(
    ///         200,
    ///         "N O S E",
    ///         vec![Header::new("Content-Type", "text/plain")],
    ///     )
    /// });
    ///
    /// // Starts the server
    /// // This is blocking
    /// # server.set_run(false);
    /// server.start();
    /// ```
    pub fn all(&mut self, handler: fn(Request) -> Response) {
        self.routes
            .push(Route::new(Method::ANY, "*".to_string(), handler));
    }

    /// Create a new route for any type of request
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
    ///     Response::new(
    ///         200,
    ///         "N O S E",
    ///         vec![Header::new("Content-Type", "text/plain")],
    ///     )
    /// });
    ///
    /// // Starts the server
    /// // This is blocking
    /// # server.set_run(false);
    /// server.start();
    /// ```
    /// Now you can make any type of request to `/nose` and it will return a 200
    pub fn any(&mut self, path: &str, handler: fn(Request) -> Response) {
        self.routes
            .push(Route::new(Method::ANY, path.to_string(), handler));
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
    /// server.every(Box::new(|req| {
    ///     // Do something with the request
    ///     // Return a `None` to continue to the next middleware / route
    ///     // Return a `Some` to send a response
    ///    None
    ///}));
    ///
    /// // Starts the server
    /// // This is blocking
    /// # server.set_run(false);
    /// server.start();
    /// ```
    pub fn every(&mut self, handler: Box<dyn Fn(&Request) -> Option<Response>>) {
        self.middleware.push(handler);
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
    ///     Response::new(
    ///         200,
    ///         "N O S E",
    ///         vec![Header::new("Content-Type", "text/plain")],
    ///     )
    /// });
    ///
    /// // Starts the server
    /// // This is blocking
    /// # server.set_run(false);
    /// server.start();
    /// ```
    pub fn route(&mut self, method: Method, path: &str, handler: fn(Request) -> Response) {
        self.routes
            .push(Route::new(method, path.to_string(), handler));
    }
}

/// Get the request method of a raw HTTP request.
fn get_request_method(raw_data: String) -> Method {
    let method_str = raw_data
        .split(' ')
        .collect::<Vec<&str>>()
        .get(0)
        .unwrap()
        .to_string();

    match &method_str[..] {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "OPTIONS" => Method::OPTIONS,
        "HEAD" => Method::HEAD,
        "PATCH" => Method::PATCH,
        "TRACE" => Method::TRACE,
        _ => Method::CUSTOM(method_str),
    }
}

/// Get the path of a raw HTTP request.
fn get_request_path(raw_data: String) -> String {
    let path_str = raw_data.split(' ').collect::<Vec<&str>>();
    if path_str.len() > 1 {
        let path = path_str[1].to_string();
        let path = path.split('?').collect::<Vec<&str>>();
        return path[0].to_string();
    }
    "".to_string()
}

// Get The Query Data of a raw HTTP request.
fn get_request_query(raw_data: String) -> Query {
    let path_str = raw_data.split(' ').collect::<Vec<&str>>();
    if path_str.len() > 1 {
        let path = path_str[1].to_string();
        let path = path.split('?').collect::<Vec<&str>>();

        if path.len() <= 1 {
            return Query::new_empty();
        }
        return Query::new(path[1]);
    }
    Query::new_empty()
}

/// Get the body of a raw HTTP request.
fn get_request_body(raw_data: String) -> String {
    let data = raw_data.split("\r\n\r\n").collect::<Vec<&str>>();

    if data.len() >= 2 {
        return data[1].to_string().trim_matches(char::from(0)).to_string();
    }
    "".to_string()
}

/// Get the headers of a raw HTTP request.
fn get_request_headers(raw_data: String) -> Vec<Header> {
    let mut headers = Vec::new();
    let spilt = raw_data.split("\r\n\r\n").collect::<Vec<&str>>();
    let raw_headers = spilt[0].split("\r\n").collect::<Vec<&str>>();

    for header in raw_headers {
        if let Some(header) = Header::from_string(header.trim_matches(char::from(0))) {
            headers.push(header)
        }
    }

    headers
}

/// Get Cookies of a raw HTTP request.
pub fn get_request_cookies(raw_data: String) -> Vec<Cookie> {
    let spilt = raw_data.split("\r\n\r\n").collect::<Vec<&str>>();
    let raw_headers = spilt[0].split("\r\n").collect::<Vec<&str>>();

    for header in raw_headers {
        if !header.starts_with("Cookie:") {
            continue;
        }

        if let Some(cookie) = Cookie::from_string(header.trim_matches(char::from(0))) {
            return cookie;
        }
    }
    Vec::new()
}

/// Get the byte size of the headers of a raw HTTP request.
fn get_header_size(raw_data: String) -> usize {
    let headers = raw_data.split("\r\n\r\n").collect::<Vec<&str>>();
    headers[0].len() + 4
}
