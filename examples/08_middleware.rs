use afire::{Header, Method, Response, Server};

// Define middleware that will be executed before the request handler.
// In afire middleware is run before the normal routes and they can ether
// send a response or pass the request to the next middleware / route.
// Just like routes, middleware is executed in reverse order that they are defined.
// So the most recently defined middleware is executed first.

// You could use middleware to Log all requests, or to check if a user is logged in, Implement ReteLimiting, add Analytics, etc.

fn main() {
    // Create a new Server instance on localhost port 8080
    let mut server: Server = Server::new("localhost", 8080);

    // Define a basic route
    server.route(Method::GET, "/", |_req| {
        Response::new(
            200,
            "Hello World!",
            vec![Header::new("Content-Type", "text/plain")],
        )
    });

    // Now to add some middleware
    // For this example we will add a middleware that will print all requests to the console
    server.every(Box::new(|req| {
        println!("[{}] {} {}", req.address, req.method, req.path);

        // Return None to forward the request to the next middleware or route
        None
    }));

    // You can now goto http://localhost:8080 you should see that the request is printed to the console
    // It should look something like this: `[127.0.0.1:xxxxx] GET /`

    println!(
        "[08] Listening on http://{}:{}",
        server.ip_string(),
        server.port
    );

    // Start the server
    // This will block the current thread
    server.start().unwrap();
}
