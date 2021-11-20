use afire::{Method, Response, ServeStatic, Server};

fn main() {
    let mut server: Server = Server::new("localhost", 8080);

    ServeStatic::new("examples/data").attach(&mut server);

    server.route(Method::GET, "/test", |_| Response::new().text("YO!"));

    server.start().unwrap();
}
