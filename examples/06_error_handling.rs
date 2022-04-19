use std::sync::RwLock;

use afire::{Method, Response, Server};

// Don't crash thread from a panic in a route
// This does not apply to middleware or the error handler itself

// afire will catch any panic in a route and return a 500 error by default
// This can be disabled by disabling the `panic_handler` feature

fn main() {
    // Create a new Server instance on localhost port 8080
    let mut server = Server::<()>::new("localhost", 8080);

    // Define a route that will panic
    server.route(Method::GET, "/panic", |_req| panic!("This is a panic!"));

    // Give the server a main page
    server.route(Method::GET, "/", |_req| {
        Response::new()
            .status(200)
            .text(r#"<a href="/panic">PANIC</a>"#)
            .header("Content-Type", "text/html")
    });

    // You can optionally define a custom error handler
    // This can be defined anywhere in the server and will take affect for all routes
    // Its like a normal route, but it will only be called if the route panics
    let errors = RwLock::new(0);
    server.error_handler(move |_req, err| {
        let mut errors = errors.write().unwrap();
        *errors += 1;

        Response::new()
            .status(500)
            .text(format!(
                "<h1>Internal Server Error #{}</h1><br>Panicked at '{}'",
                errors, err
            ))
            .header("Content-Type", "text/html")
    });

    // You can now goto http://localhost:8080/panic
    // This will cause the route to panic and return a 500 error

    println!("[06] Listening on http://{}:{}", server.ip, server.port);

    // Start the server
    // This will block the current thread
    server.start().unwrap();
}
