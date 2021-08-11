// Import STD libraries
use std::io::prelude::*;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::str;

// Import local files

use super::header::*;
use super::other::*;

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
    pub middleware: Vec<fn(&Request) -> Option<Response>>,

    /// Run server
    ///
    /// Really just for testing.
    run: bool,

    /// Headders automatically added to every response.
    default_headers: Option<Vec<Header>>,
}

/// Implamantaions for Server
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

        // If the ip is localhost, use the loopback ip
        if raw_ip == "localhost" {
            raw_ip = "127.0.0.1";
        }

        // Parse the ip to an array
        let splitted_ip: Vec<&str> = raw_ip.split('.').collect();

        if splitted_ip.len() != 4 {
            panic!("Invalid Server IP");
        }
        for i in 0..4 {
            let octet: u8 = splitted_ip[i].parse::<u8>().expect("Invalid Server IP");
            ip[i] = octet;
        }

        Server {
            port: port,
            ip: ip,
            routes: Vec::new(),
            run: true,
            default_headers: Some(vec![Header::new("Powerd-By", "afire")]),
            middleware: Vec::new(),
        }
    }

    /// Start the server.
    ///
    /// Will be blocking.
    ///
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Server, Response, Header};
    ///
    /// // Starts a server for localhost on port 8080
    /// let mut server: Server = Server::new("localhost", 8080);
    ///
    /// // Define a route
    /// server.get("/", |req| {
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
            if self.default_headers.is_some() {
                for header in self.default_headers.as_ref().unwrap() {
                    res.headers.push(Header::copy(header));
                }
            }

            // Add content-length header to response
            res.headers
                .push(Header::new("Content-Length", &res.data.len().to_string()));

            // Convert the response to a string
            let response = format!(
                "HTTP/1.1 {} OK\r\n{}\r\n\r\n{}",
                res.status,
                headers_to_string(res.headers),
                res.data
            );

            // Send the response
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    }

    /// Handel a connection to the server
    fn handle_connection(&self, mut stream: &TcpStream) -> Response {
        // Init Buffer
        let mut buffer = [0; 1024];

        // Read stream into buffer
        stream.read(&mut buffer).unwrap();

        let stream_string = str::from_utf8(&buffer).expect("Error parsing buffer data");

        // Make Request Object
        let req_method = get_request_method(stream_string.to_string());
        let req_path = get_request_path(stream_string.to_string());
        let body = get_request_body(stream_string.to_string());
        let headers = get_request_headers(stream_string.to_string());
        let req = Request::new(req_method, &req_path, headers, body);

        // Use middleware to handle request
        // If middleware returns a `None`, the request will be handled by the routes
        for middleware in self.middleware.iter().rev() {
            match (middleware)(&req) {
                None => (),
                Some(res) => return res,
            }
        }

        // Loop through all routes and check if the request matches
        for route in self.routes.iter().rev() {
            if (&req.method == &route.method || route.method == Method::ANY)
                && (req.path == route.path || req_path == "*")
            {
                return (route.handler)(req);
            }
        }

        // If no route was found, return a default 404
        return Response::new(
            404,
            "Not Found",
            vec![Header::new("Content-Type", "text/plain")],
        );
    }

    /// Keep a server from starting
    ///
    /// Only used for testing
    ///
    /// It would be a really dumb idea to use this
    pub fn set_run(&mut self, run: bool) {
        self.run = run;
    }

    /// Create a new route the runs for all methods
    ///
    /// May be useful for a 404 page as the most recently defined route takes priority
    /// so by defining this route first it would trigger if nothing else matches
    pub fn all(&mut self, handler: fn(Request) -> Response) {
        self.routes.push(Route {
            method: Method::ANY,
            path: "*".to_string(),
            handler: handler,
        });
    }

    /// Create a new route for any type of request
    pub fn any(&mut self, path: &str, handler: fn(Request) -> Response) {
        self.routes.push(Route {
            method: Method::ANY,
            path: path.to_string(),
            handler: handler,
        });
    }

    /// Add a new middleware to the server
    ///
    /// You will have access to the request object
    /// But will not be able to access the response
    pub fn every(&mut self, handler: fn(&Request) -> Option<Response>) {
        self.middleware.push(handler);
    }

    /// Create a new route for get requests
    pub fn get(&mut self, path: &str, handler: fn(Request) -> Response) {
        self.routes.push(Route {
            method: Method::GET,
            path: path.to_string(),
            handler: handler,
        });
    }
}

/// Get the request method of a raw HTTP request.
fn get_request_method(raw_data: String) -> Method {
    let method_str = raw_data
        .split(' ')
        .collect::<Vec<&str>>()
        .iter()
        .next()
        .unwrap()
        .to_string();

    return match &method_str[..] {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "OPTIONS" => Method::OPTIONS,
        "HEAD" => Method::HEAD,
        "PATCH" => Method::PATCH,
        "TRACE" => Method::TRACE,
        _ => Method::CUSTOM(method_str),
    };
}

/// Get the path of a raw HTTP request.
fn get_request_path(raw_data: String) -> String {
    let path_str = raw_data.split(" ").collect::<Vec<&str>>();
    path_str[1].to_string()
}

/// Get the body of a raw HTTP request.
fn get_request_body(raw_data: String) -> String {
    raw_data
        .split("\r\n\r\n")
        .collect::<Vec<&str>>()
        .iter()
        .next()
        .unwrap()
        .to_string()
}

/// Get the headers of a raw HTTP request.
fn get_request_headers(raw_data: String) -> Vec<Header> {
    let mut headers = Vec::new();
    let raw_headers = raw_data.split("\r\n").collect::<Vec<&str>>();

    for header in raw_headers {
        match Header::from_string(header) {
            Some(header) => headers.push(header),
            None => (),
        }
    }

    headers
}
