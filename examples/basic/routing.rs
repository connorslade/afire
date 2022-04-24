use afire::{Method, Response, Server};

use crate::Example;

// Introduce the way routing works in afire

// In afire the newest routes take priority over other routes.
// This means that if you have two routes that could run for a request
// the one defined last will run.

// To explain this better I will label the routes with numbers to represent their priority.
// Higher priority numbers will run first
// Note: In the afire backend code thare is no priority number its just the order in which they are defined
pub struct Routing;

impl Example for Routing {
    fn name(&self) -> &'static str {
        "routing"
    }

    fn exec(&self) {
        // Create a new Server instance on localhost port 8080
        let mut server: Server = Server::new("localhost", 8080);

        // Define 404 page
        // This route will run for all requests but because any other route
        // will take priority it will only run when no other route is defined.
        /* PRIO 0 */
        server.route(Method::ANY, "*", |_req| {
            Response::new()
                .status(404)
                .text("The page you are looking for does not exist :/")
                .header("Content-Type", "text/plain")
        });

        // Define a route
        // As this is defined last, it will take a higher priority
        /* PRIO 1 */
        server.route(Method::GET, "/", |_req| {
            Response::new()
                .status(200)
                .text("Hello World!")
                .header("Content-Type", "text/plain")
        });

        // Now goto http://localhost:8080/ and you should see "Hello World"
        // But if you go to http://localhost:8080/somthing-else you should see the 404 page

        // Start the server
        // This will block the current thread
        server.start().unwrap();
    }
}
