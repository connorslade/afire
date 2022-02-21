use std::sync::atomic::{AtomicU32, Ordering};

use afire::prelude::*;

struct Log(AtomicU32);

impl Middleware for Log {
    fn pre(&self, _req: Request) -> MiddleRequest {
        self.0
            .fetch_update(Ordering::Release, Ordering::Relaxed, |x| Some(x + 1))
            .unwrap();
        println!("{}", self.0.load(Ordering::Acquire));

        std::thread::sleep(std::time::Duration::from_secs(10));

        MiddleRequest::Continue
    }
}

impl Log {
    fn new() -> Self {
        Self(AtomicU32::new(0))
    }
}

fn main() {
    let mut server: Server = Server::new("localhost", 8818);

    Log::new().attach(&mut server);

    server.route(Method::GET, "/", |_req| {
        Response::new()
            .status(200)
            .reason("OK!")
            .text("Hi :P")
            .header("Content-Type", "text/plain")
    });

    server.start_threaded(10).unwrap();
    // server.start().unwrap();
}
