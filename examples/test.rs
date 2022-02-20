use afire::prelude::*;

struct Log(u32);

impl Middleware for Log {
    fn pre(&mut self, _req: Request) -> MiddleRequest {
        self.0 += 1;
        println!("{}", self.0);

        MiddleRequest::Continue
    }
}

fn main() {
    let mut server: Server = Server::new("localhost", 8818);

    Log(0).attach(&mut server);

    server.route(Method::GET, "/", |_req| {
        std::thread::sleep(std::time::Duration::from_secs(10));
        Response::new()
            .status(200)
            .reason("OK!")
            .text("Hi :P")
            .header("Content-Type", "text/plain")
    });

    server.start_threaded(10).unwrap();
    // server.start().unwrap();
}
