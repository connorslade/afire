use afire::{extension::RateLimiter, Content, Method, Middleware, Response, Server};

use crate::Example;

// Use some of afire's built-in middleware to log requests.
pub struct RateLimit;

impl Example for RateLimit {
    fn name(&self) -> &'static str {
        "rate_limit"
    }

    fn exec(&self) {
        // Create a new Server instance on localhost port 8080
        let mut server = Server::<()>::new("localhost", 8080);

        // Define a handler for GET "/"
        server.route(Method::GET, "/", |_req| {
            Response::new()
                .status(200)
                .text("Hello World!")
                .content(Content::TXT)
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
                        .content(Content::TXT),
                )
            }))
            // Attach to the server
            .attach(&mut server);

        // Now if you goto http://localhost:8080/ and reload a bunch of times,
        // you'll see the rate limiter kicking in.

        // Start the server
        // This will block the current thread
        server.start().unwrap();
    }
}
