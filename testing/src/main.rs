use afire::afire::*;
use std::fs;

fn main() {
    let mut server: Server = Server::new("localhost", 1234);

    // Define a handler for GET "/"
    server.get("/", |_req| {
        Response::new(
            200,
            "Hi :P",
            vec![Header::new("Content-Type", "text/plain")],
        )
    });

    // Define a handler for GET "/nose"
    server.get("/nose", |_req| {
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
    server.get("/pi", |_req| {
        Response::new(
            200,
            // Html stored as txt because yes
            &fs::read_to_string("data/index.txt").unwrap(),
            vec![Header::new("Content-Type", "text/html")],
        )   
    });

    // Start the server
    server.start();
}