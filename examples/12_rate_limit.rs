use afire::{Header, Method, RateLimiter, Response, Server};

// Use some of afire's built-in middleware to log requests.

fn main() {
    // Create a new Server instance on localhost port 8080
    let mut server: Server = Server::new("localhost", 8080);

    // Define a handler for GET "/"
    server.route(Method::GET, "/", |_req| {
        Response::new(
            200,
            "Hello World!",
            vec![Header::new("Content-Type", "text/plain")],
        )
    });

    // Add the rate limiter middleware
    // For this example, we'll limit requests to 1 every 2 seconds
    // RateLimiter::attach(&mut server, RateLimiter::new(1, 2));

    // We can even add a custom handler for when the rate limiter is exceeded
    RateLimiter::attach(
        &mut server,
        RateLimiter::new_handler(
            1,
            2,
            Box::new(|_req| {
                Some(Response::new(
                    429,
                    "AHHHH!!! Too Many Requests",
                    vec![Header::new("Content-Type", "text/plain")],
                ))
            }),
        ),
    );

    // Now if you goto http://localhost:8080/ and reload a bunch of times,
    // you'll see the rate limiter kicking in.

    println!(
        "[11] Listening on http://{}:{}",
        server.ip_string(),
        server.port
    );

    // Start the server
    // This will block the current thread
    server.start().unwrap();
}
