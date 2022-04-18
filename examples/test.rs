use afire::{extension::ServeStatic, Method, Middleware, Response, Server};

fn main() {
    let mut server: Server = Server::new("localhost", 8080);

    server.route(Method::GET, "/index.html", |_req| {
        Response::new().text("Hi :P")
    });
    ServeStatic::new("examples/data").attach(&mut server);

    server.start().unwrap();
}
