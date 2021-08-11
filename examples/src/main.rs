use afire::*;
use std::fs;

fn main() {
    let mut server: Server = Server::new("localhost", 1234);

    // Define a handler for GET "/"
    server.route(Method::GET, "/", |_req| {
        Response::new(
            200,
            "Hi :P",
            vec![Header::new("Content-Type", "text/plain")],
        )
    });

    // Define a handler for GET "/nose"
    server.route(Method::GET, "/nose", |_req| {
        Response::new(
            200,
            "N O S E",
            vec![Header::new("Content-Type", "text/plain")],
        )
    });

    // Define a handler for ANY "/hi"
    server.route(Method::GET, "/hi", |_req| {
        Response::new(
            200,
            "<h1>Hello, How are you?</h1>",
            vec![Header::new("Content-Type", "text/html")],
        )
    });

    // Serve a file
    server.route(Method::GET, "/pi", |_req| {
        Response::new(
            200,
            &fs::read_to_string("data/index.txt").unwrap(),
            vec![Header::new("Content-Type", "text/html")],
        )
    });

    // Start the server
    server.start();
}
