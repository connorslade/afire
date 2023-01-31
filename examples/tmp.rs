// use afire::extension::Logger;
use afire::prelude::*;

fn main() {
    let mut server = Server::<()>::new("localhost", 8080);
    // Logger::new().attach(&mut server);

    server.route(Method::POST, "/upload", |req| {
        println!("Receved {} bytes", req.body.len());
        Response::new()
            .text(format!("Receved {} bytes", req.body.len()))
            .header("Access-Control-Allow-Origin", "https://gogo.mango")
    });

    server.route(Method::GET, "/", |_| Response::new().text("Hello World!"));

    server.start_threaded(5).unwrap();
}
