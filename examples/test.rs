use afire::{
    middleware::{MiddleResponse, Middleware},
    Header, Logger, Method, Response, Server,
};

struct Log(usize);

impl Middleware for Log {
    fn post(&mut self, res: Response) -> MiddleResponse {
        self.0 += 1;

        MiddleResponse::Add(
            res.header(Header::new("Middleware", "Active"))
                .header(Header::new("Count", self.0)),
        )
    }
}

impl Log {
    fn new() -> Log {
        Log(0)
    }
}

fn main() {
    let mut server: Server = Server::new("localhost", 8080);

    server.route(Method::GET, "/", |_req| {
        Response::new()
            .status(200)
            .reason("OK!")
            .text("Hi :P")
            .header(Header::new("Content-Type", "text/plain"))
    });

    Log::new().attach(&mut server);
    Logger::new().attach(&mut server);

    server.start().unwrap();
}
