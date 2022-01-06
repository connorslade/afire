use afire::{Header, Method, Middleware, Response, Server};

struct Log(usize);

impl Middleware for Log {
    fn post(&mut self, res: Response) -> Response {
        self.0 += 1;

        res.header(Header::new("Middleware", "Active"))
            .header(Header::new("Count", self.0))
    }
}

impl Log {
    fn new() -> Log {
        Log(0)
    }
}

fn main() {
    let mut server: Server = Server::new("localhost", 8080);

    server.route(Method::GET, "/", |req| {
        Response::new()
            .status(200)
            .reason("OK!")
            .text("Hi :P")
            .header(Header::new("Content-Type", "text/plain"))
    });

    Log::new().attach(&mut server);

    server.start().unwrap();
}
