use afire::{
    middleware::{MiddleResult, Middleware},
    Content, Header, Method, Request, Response, Server,
};

use crate::Example;

// In afire Middleware is a trait that is implemented and can modify Requests and Response before and after Routes
// You can use Middleware to Log Requests, Ratelimit Requests, add Analytics, etc.
// The Middleware functions for this are `pre` and `post` for before and after the routes, there is also `end` which is called after the response is sent to the client
//
// There are two types of hooks: raw and non-raw.
// The raw hooks are passed a Result, and their default implementation calls the non-raw hooks if the Result is Ok.
// This allows you to handle errors (like page not found), while maintaining a clean API for middleware that doesn't need to handle errors.
//
// In the different middleware hooks you can return a MiddleResult, which is an enum with 3 variants:
// - Continue: Continue to the next middleware or route
// - Abort: Stop the middleware chain
// - Send: Immediately send this response to the client and stop the middleware chain
//
// For more info, checkout the documentation for Middleware here: https://docs.rs/afire/latest/afire/middleware/trait.Middleware.html

// Lets make a Middleware that will log the request to the console
// And to show how to modify the response, we will add a header to the response

struct Log;

// Now we will Implement Middleware for Log
impl Middleware for Log {
    // Redefine the `pre` function
    // (Runs before Routes)
    fn pre(&self, req: &mut Request) -> MiddleResult {
        // Print some info
        println!("[{}] {} {}", req.address.ip(), req.method, req.path);

        // Continue to forward the request to the next middleware or route
        MiddleResult::Continue
    }

    // Lets also modify the outgoing response by adding a header
    fn post(&self, _req: &Request, res: &mut Response) -> MiddleResult {
        res.headers.push(Header::new("X-Example", "Middleware"));
        MiddleResult::Continue
    }
}

pub struct MiddlewareExample;

impl Example for MiddlewareExample {
    fn name(&self) -> &'static str {
        "middleware"
    }

    fn exec(&self) {
        // Create a new Server instance on localhost port 8080
        let mut server = Server::<()>::new("localhost", 8080);

        // Define a basic route
        server.route(Method::GET, "/", |_req| {
            Response::new().text("Hello World!").content(Content::TXT)
        });

        // Here is where we will attach our Middleware to the Server
        // This is super easy
        Log.attach(&mut server);

        // You can now goto http://localhost:8080 you should see that the request is printed to the console
        // It should look something like this: `[127.0.0.1] GET `

        // Start the server
        // This will block the current thread
        server.start().unwrap();
    }
}
