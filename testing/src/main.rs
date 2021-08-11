use afire::*;
use std::fs;

fn main() {
    let mut server: Server = Server::new("localhost", 1234);

    // Define a catch-all handler
    // This will be called when no other handlers match
    server.all(|_req| {
        Response::new(
            404,
            "Not Found",
            vec![Header::new("Content-Type", "text/plain")],
        )
    });

    // Define a handler for route "/"
    server.route(Method::GET, "/", |_req| {
        Response::new(
            200,
            "Hi :P",
            vec![Header::new("Content-Type", "text/plain")],
        )
    });

    // Define a handler for route "/nose"
    server.route(Method::GET, "/nose", |_req| {
        Response::new(
            200,
            "N O S E",
            vec![Header::new("Content-Type", "text/plain")],
        )
    });

    // Define a handler for ANY "/hi"
    server.any("/hi", |_req| {
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
            // Html stored as txt because yes
            &fs::read_to_string("data/index.txt").unwrap(),
            vec![Header::new("Content-Type", "text/html")],
        )
    });

    server.every(|req| {
        println!("req: {:?}", req);
        None
    });

    // Start the server
    server.start();
}
