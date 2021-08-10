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
        Response::new(
            200,
            "Hi :P",
            vec![Header::new("Content-Type", "text/plain")],
        )
    });
    server.start();
}

pub struct Server {
    pub port: u16,
    pub ip: [u8; 4],

    pub routes: Vec<Route>,

    // Optional
    default_headers: Option<Vec<Header>>,
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

pub struct Header {
    name: String,
    value: String,
}

pub struct Request {
    pub method: Method,
    pub path: String,
    pub headers: Vec<Header>,
    pub body: Vec<u8>,
}

pub struct Response {
    pub status: u16,
    pub data: String,
    pub headers: Vec<Header>,
}

impl Server {
    fn new(mut raw_ip: &str, port: u16) -> Server {
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
            default_headers: Some(vec![Header::new("Powerd-By", "afire")]),
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
            for header in self.default_headers.as_ref().unwrap() {
                res.headers.push(Header::copy(header));
            }
            res.headers
                .push(Header::new("Content-Length", &res.data.len().to_string()));

            let response = format!(
                "HTTP/1.1 {} OK\n{}\n\n{}",
                res.status,
                headers_to_string(res.headers),
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

        // DEBUG
        println!("{}", String::from_utf8_lossy(&buffer[..]));

        let stream_string = str::from_utf8(&buffer).expect("Error parseing buffer data");

        // Loop through all routes and check if the request matches
        for route in self.routes.iter().rev() {
            if &get_request_method(stream_string.to_string()) == &route.method
                && get_request_path(stream_string.to_string()) == route.path
            {
                let req = Request::new(Method::GET, "/", Vec::new(), Vec::new());
                return (route.handler)(req);
            }
        }
        return Response::new(
            404,
            "Not Found",
            vec![Header::new("Content-Type", "text/plain")],
        );
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
    pub fn new(status: u16, data: &str, headers: Vec<Header>) -> Response {
        Response {
            status,
            data: data.to_string(),
            headers: headers,
        }
    }
}

impl Request {
    fn new(method: Method, path: &str, headers: Vec<Header>, body: Vec<u8>) -> Request {
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

impl Header {
    fn new(name: &str, value: &str) -> Header {
        Header {
            name: name.to_string(),
            value: value.to_string(),
        }
    }

    fn copy(header: &Header) -> Header {
        Header {
            name: header.name.clone(),
            value: header.value.clone(),
        }
    }

    fn to_string(&self) -> String {
        format!("{}: {}", self.name, self.value)
    }

    fn from_string(header: &str) -> Option<Header> {
        let splitted_header: Vec<&str> = header.split(':').collect();
        if splitted_header.len() != 2 {
            return None;
        }
        Some(Header {
            name: splitted_header[0].trim().to_string(),
            value: splitted_header[1].trim().to_string(),
        })
    }
}

fn headers_to_string(headers: Vec<Header>) -> String {
    let headers_string: Vec<String> = headers.iter().map(|header| header.to_string()).collect();
    format!("{}", headers_string.join("\n"))
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

fn get_request_path(raw_data: String) -> String {
    let path_str = raw_data.split(" ").collect::<Vec<&str>>();
    path_str[1].to_string()
}
