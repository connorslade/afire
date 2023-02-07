use afire::{HeaderType, Method, Response, Server};

use crate::Example;

// Create a new basic server
// It will just serve the text "Hi :P:
pub struct Basic;

impl Example for Basic {
    fn name(&self) -> &'static str {
        "basic"
    }

    fn exec(&self) {
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
                .header(HeaderType::Date, "today")
                .content(afire::Content::TXT)
        });

        // Start the server
        // This will block the current thread
        server.start().unwrap();
    }
}
