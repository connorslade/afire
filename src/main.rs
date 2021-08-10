// use std::fmt::Write;
use std::io::prelude::*;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::str;

fn main() {
    let mut server: Server = Server::new("localhost", 1234);
    server.get("/", |_req| {
        // println!("Hi :P");
        Response::new(200, "Hi :P", vec!["Content-Type: text/plain"])
    });
    server.start();
}

pub struct Server {
    pub port: u16,
    pub ip: [u8; 4],

    pub routes: Vec<Route>,
}

pub struct Route {
    method: Method,
    path: String,
    handler: fn(Request) -> Response,
}

pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    OPTIONS,
    HEAD,
    PATCH,
    TRACE,
    CUSTOM(String),
}

pub struct Request {
    pub method: Method,
    pub path: String,
    pub headers: Vec<String>,
    pub body: Vec<u8>,
}

pub struct Response {
    pub status: u16,
    pub data: String,
    pub headers: Vec<String>,
}

impl Server {
    fn new(raw_ip: &str, port: u16) -> Server {
        let mut ip: [u8; 4] = [0; 4];

        // If the ip is localhost, use the loopback ip
        if raw_ip == "localhost" {
            return Server {
                port: port,
                ip: [127, 0, 0, 1],
                routes: Vec::new(),
            };
        }

        // Parse the ip to an array
        let splitted_ip: Vec<&str> = raw_ip.split('.').collect();
        if splitted_ip.len() != 4 {
            panic!("Invalid Server IP");
        }
        for i in 0..3 {
            let octet: u8 = splitted_ip[i].parse::<u8>().expect("Invalid Server IP");
            ip[i] = octet;
        }

        Server {
            port: port,
            ip: ip,
            routes: Vec::new(),
        }
    }

    fn start(&self) {
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

            // Get the reponse from the handler
            // Uses the most recently defined route that matches the request
            let mut res = self.handle_connection(&stream);
            res.headers.push(format!("Content-Length: {}", res.data.len()));

            let response = format!(
                "HTTP/1.1 {} OK\n{}\n\n{}",
                res.status,
                res.headers.join("\n"),
                res.data
            );

            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    }

    fn handle_connection(&self, mut stream: &TcpStream) -> Response {
        // Init Buffer
        let mut buffer = [0; 1024];

        stream.read(&mut buffer).unwrap();

        let stream_string = str::from_utf8(&buffer).expect("Error parseing buffer data");

        // Loop through all routes and check if the request matches
        for route in self.routes.iter().rev() {
            if &get_request_method(stream_string.to_string()) == &route.method && "/" == route.path
            {
                let req = Request::new(Method::GET, "/", Vec::new(), Vec::new());
                return (route.handler)(req);
            }
        }
        return Response::new(404, "Not Found", Vec::new());
    }

    fn get(&mut self, path: &str, handler: fn(Request) -> Response) {
        self.routes.push(Route {
            method: Method::GET,
            path: path.to_string(),
            handler: handler,
        });
    }
}

impl Response {
    /// Quick and easy way to create a response.
    pub fn new(status: u16, data: &str, headers: Vec<&str>) -> Response {
        let new_headers: Vec<String> = headers.iter().map(|header| header.to_string()).collect();
        Response {
            status,
            data: data.to_string(),
            headers: new_headers,
        }
    }
}

impl Request {
    fn new(method: Method, path: &str, headers: Vec<String>, body: Vec<u8>) -> Request {
        Request {
            method,
            path: path.to_string(),
            headers,
            body,
        }
    }
}

impl PartialEq for Method {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

/// Get the request method of a raw HTTP request.
fn get_request_method(raw_data: String) -> Method {
    let method_str = raw_data
        .split(" ")
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
