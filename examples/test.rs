use afire::*;

fn main() {
    let mut server: Server = Server::new("localhost", 8081);

    server.route(Method::GET, "/", |req| {
        Response::new()
            .status(200)
            .text(String::from_utf8_lossy(&req.body.clone()))
            .header(Header::new("Content-Type", "text/plain"))
    });

    server.start().unwrap();
}
