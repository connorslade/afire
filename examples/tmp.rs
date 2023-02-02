use std::fs::{self, File};

use afire::trace::Level;
// use afire::extension::Logger;
use afire::trace::set_log_level;
use afire::{prelude::*, Middleware};

const PATH: &str = r#"/home/connorslade/Downloads/"#;

fn main() {
    let mut server = Server::<()>::new("localhost", 8080);
    set_log_level(Level::Debug);
    // Logger::new().attach(&mut server);

    server.route(Method::POST, "/upload", |req| {
        println!("Receved {} bytes", req.body.len());
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

    Test.attach(&mut server);
    server.start_threaded(5).unwrap();
}

struct Test;

impl Middleware for Test {
    fn pre(&self, req: &mut Request) -> MiddleResult {
        println!("PROCESSING {}", req.path);

        if req.path.contains("hello") {
            return MiddleResult::Abort(Response::new().text("Hello World"));
        }

        MiddleResult::Continue
    }
}
