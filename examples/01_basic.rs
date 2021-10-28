use afire::{Header, Method, Response, Server};

// Create a new basic server
// It will just serve the text "Hi :P:

fn main() {
    // Create a new Server instance on localhost port 8080
    let mut server: Server = Server::new("localhost", 8080);

    // Define a handler for GET "/"
    server.route(Method::GET, "/", |_req| {
        Response::new(
            200,
            "Hi :P",
            vec![Header::new("Content-Type", "text/plain")],
        )
    });

    println!(
        "[01] Listening on http://{}:{}",
        server.ip_string(),
        server.port
    );

    // Start the server
    // This will block the current thread
    server.start().unwrap();
}
