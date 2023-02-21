use std::fs::{self, File};

use afire::{extension::Date, extension::Logger, prelude::*, trace::set_log_level, trace::Level};

// File to download
const PATH: &str = r#"..."#;

fn main() {
    let mut server = Server::<()>::new("localhost", 8080);
    set_log_level(Level::Debug);
    Logger::new().attach(&mut server);

    server.route(Method::POST, "/upload", |req| {
        println!("Received {} bytes", req.body.len());
        Response::new().bytes(&req.body)
    });

    server.route(Method::GET, "/download", |_| {
        let data = fs::read(PATH).unwrap();
        Response::new().bytes(&data)
    });

    server.route(Method::GET, "/download-stream", |_| {
        let stream = File::open(PATH).unwrap();
        Response::new().stream(stream)
    });

    server.route(Method::GET, "/", |req| {
        let mango = req.socket.lock().unwrap();
        let user_agent = req.headers.get(HeaderType::UserAgent).unwrap();
        println!("{}", mango.peer_addr().unwrap());
        Response::new().text(user_agent).content(Content::TXT)
    });

    server.route(Method::ANY, "/panic", |_| panic!("panic!"));

    Test.attach(&mut server);
    Date.attach(&mut server);
    server.start_threaded(5).unwrap();
}

struct Test;

impl Middleware for Test {
    fn pre(&self, req: &mut Request) -> MiddleResult {
        if req.path.contains("hello") {
            println!("Pre: {}", req.path);
            return MiddleResult::Send(Response::new().text("Intercepted"));
        }

        MiddleResult::Continue
    }

    fn post(&self, req: &Request, _res: &mut Response) -> MiddleResult {
        if req.path.contains("hello") {
            println!("Post: {}", req.path);
        }

        MiddleResult::Continue
    }

    fn end(&self, req: &Request, _res: &Response) {
        if req.path.contains("hello") {
            println!("End: {}", req.path);
        }
    }
}
