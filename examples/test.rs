use afire::*;

fn main() {
    let mut server: Server = Server::new("localhost", 8081);

    server.route(Method::GET, "/", |req| {
        println!("{:?}", String::from_utf8_lossy(&req.raw_data.clone()));

        Response::new()
            .status(200)
            .text(req.body_string().unwrap())
            .header(Header::new("Content-Type", "text/plain"))
    });

    server.start().unwrap();
}
