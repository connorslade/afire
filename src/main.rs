use std::io::prelude::*;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::TcpListener;

fn main() {
    let mut server: Server = Server::new("localhost", 1234);
    server.get("/", |req, res| {
        println!("Hi :P");
        Response::new(200, "Hi :P", vec!["Content-Type: application/json"])
    });
    server.start();
}

pub struct Server {
    pub port: u16,
    pub ip: [u8; 4],

    pub routes: Vec<Route>,
}

pub struct Route {
    method: Methood,
    path: String,
    handler: fn(Request, Response) -> Response,
}

pub enum Methood {
    GET,
    POST,
    PUT,
    DELETE,
    OPTIONS,
    HEAD,
    PATCH,
    TRACE,
}

pub struct Request {
    pub method: Methood,
    pub path: String,
    pub headers: Vec<(String, String)>,
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
        let splitted_ip: Vec<&str> = raw_ip.split(".").collect();
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

        for stream in listener.incoming() {
            let mut stream = stream.unwrap();

            println!("Connection established!");
            let mut buffer = [0; 1024];

            stream.read(&mut buffer).unwrap();

            println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
        }
    }

    fn get(&mut self, path: &str, handler: fn(Request, Response) -> Response) {
        self.routes.push(Route {
            method: Methood::GET,
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
