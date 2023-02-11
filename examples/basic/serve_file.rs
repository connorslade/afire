use afire::{Content, Method, Response, Server, Status};
use std::fs::File;

use crate::Example;

// You can run this example with `cargo run --example basic -- serve_file`

// Serve a local file
// On each request, the server will read the file and send it to the client.
// Usually it is preferred to use the ServeStatic middleware for this

pub struct ServeFile;

impl Example for ServeFile {
    fn name(&self) -> &'static str {
        "serve_file"
    }

    fn exec(&self) {
        // Create a new Server instance on localhost port 8080
        let mut server = Server::<()>::new("localhost", 8080);

        // Define a handler for GET "/"
        server.route(Method::GET, "/", |_req| {
            // Try to open a file
            match File::open("examples/basic/data/index.html") {
                // If its found send it as response
                // Because we used File::open and not fs::read, we can use the stream method to send the file in chunks
                // This is more efficient than reading the whole file into memory and then sending it
                Ok(content) => Response::new().stream(content).content(Content::HTML),

                // If the file is not found, send a 404 response
                Err(_) => Response::new()
                    .status(Status::NotFound)
                    .text("Not Found :/")
                    .content(Content::TXT),
            }
        });

        // View the file at http://localhost:8080

        // Start the server
        // This will block the current thread
        server.start().unwrap();
    }
}
