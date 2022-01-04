use afire::{Header, Method, RateLimiter, Response, Server};

// Use some of afire's built-in middleware to log requests.

fn main() {
    // Create a new Server instance on localhost port 8080
    let mut server: Server = Server::new("localhost", 8080);

    // Define a handler for GET "/"
    server.route(Method::GET, "/", |_req| {
        Response::new()
            .status(200)
            .text("Hello World!")
            .header(Header::new("Content-Type", "text/plain"))
    });

    // For this example, we'll limit requests to 1 every 2 seconds

    // Make a new Ratelimater
    // Default Limit is 10
    // Default Timeout is 60 sec
    RateLimiter::new()
        // Overide the Limit to 1
        .limit(1)
        // Overide the timeout to 2
        .timeout(2)
        // Overide thge Handler
        .handler(Box::new(|_req| {
            Some(
                Response::new()
                    .status(429)
                    .text("AHHHH!!! Too Many Requests")
                    .header(Header::new("Content-Type", "text/plain")),
            )
        }))
        // Attach to the server
        .attach(&mut server);

    // Now if you goto http://localhost:8080/ and reload a bunch of times,
    // you'll see the rate limiter kicking in.

    println!("[11] Listening on http://{}:{}", server.ip, server.port);

    // Start the server
    // This will block the current thread
    server.start().unwrap();
}
