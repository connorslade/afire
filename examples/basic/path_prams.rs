use afire::{Content, Method, Response, Server};

use crate::Example;

// You can run this example with `cargo run --example basic -- path_params`

// Use Path params to send data through a route path
// You can also add `*` segments to match with any text
pub struct PathParam;

impl Example for PathParam {
    fn name(&self) -> &'static str {
        "path_params"
    }

    fn exec(&self) {
        // Create a new Server instance on localhost port 8080
        let mut server: Server = Server::<()>::new("localhost", 8081);

        // Define a handler for GET "/greet/{name}"
        // This will handel requests with anything where the {name} is
        // This includes "/greet/bob", "/greet/fin"
        server.route(Method::GET, "/greet/{name}", |req| {
            // Get name path param
            let name = req.param("name").unwrap();

            // Make a nice Message to send
            let message = format!("Hello, {}", name);

            // Send Response
            Response::new().text(message).content(Content::TXT)
        });

        // Define a greet route for Darren because he is very cool
        // This will take priority over the other route as it is defined after
        server.route(Method::GET, "/greet/Darren/", |_req| {
            Response::new().text("Hello, Darren. You are very cool")
        });

        // Start the server
        // This will block the current thread
        server.start().unwrap();
    }
}
