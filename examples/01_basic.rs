use afire::{Method, Response, Server};

// Create a new basic server
// It will just serve the text "Hi :P:

fn main() {
    // Create a new Server instance on localhost port 8080
    let mut server = Server::<()>::new("localhost", 8080);

    // Define a handler for GET "/"
    server.route(Method::GET, "/", |_req| {
        Response::new()
            // By default the status is 200
            // You can also define it yourself with the status method
            .status(200)
            // By default the reason phrase is derived from the status
            .reason("OK!")
            // Although is is named `text` it takes any type that impls Display
            // So for example numbers work too
            .text("Hi :P")
            .header("Content-Type", "text/plain")
    });

    println!("[01] Listening on http://{}:{}", server.ip, server.port);

    // Start the server
    // This will block the current thread
    server.start().unwrap();
}
