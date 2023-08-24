use afire::{extension, Header, Middleware, Response, Server, Status};

use crate::Example;

// Serve static files from a directory
// afire middleware makes this *easy*
pub struct ServeStatic;

const STATIC_DIR: &str = "examples/basic/data";
const STATIC_PATH: &str = "/";

impl Example for ServeStatic {
    fn name(&self) -> &'static str {
        "serve_static"
    }

    fn exec(&self) {
        // Create a new Server instance on localhost port 8080
        let mut server = Server::<()>::new("localhost", 8080);

        // Make a new static file server with a path
        extensions::ServeStatic::new(STATIC_DIR)
            // The middleware priority is by most recently defined.
            // The middleware function takes 3 parameters: the request, the response, and weather the file was loaded successfully.
            // In your middleware you can modify the response and the bool.
            .middleware(|req, res, _suc| {
                // Print path served
                println!("Served: {}", req.path);
                // Return none to not mess with response
                // Or in this case add a header and pass through the success value
                res.headers.push(Header::new("X-Static", "true"));
            })
            // Function that runs when no file is found to serve
            // This will run before middleware
            .not_found(|_req, _dis| {
                Response::new()
                    .status(Status::NotFound)
                    .text("Page Not Found!")
            })
            // Add an extra mime type to the server
            // It has a lot already
            .mime_type("key", "value")
            // Set serve path
            .path(STATIC_PATH)
            // Attach the middleware to the server
            .attach(&mut server);

        // View the file at http://localhost:8080
        // You should also see a favicon in the browser tab

        // Start the server
        // This will block the current thread
        server.start().unwrap();
    }
}
