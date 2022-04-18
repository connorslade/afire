use afire::{extension::RequestId, Method, Middleware, Response, Server};

fn main() {
    let mut server: Server = Server::new("localhost", 8080);

    server.route(Method::GET, "/", |req| {
        Response::new().text(req.header("X-REQ-ID").unwrap())
    });
    RequestId::new("X-REQ-ID").attach(&mut server);

    server.start().unwrap();
}
