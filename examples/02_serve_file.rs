use afire::{Method, Response, Server};
use std::fs;

// Serve a local file
// On each request, the server will read the file and send it to the client.

fn main() {
    // Create a new Server instance on localhost port 8080
    let mut server: Server = Server::new("localhost", 8080);

    // Define a handler for GET "/"
    server.route(Method::GET, "/", |_req| {
        // Try to read File
        match fs::read("examples/data/index.html") {
            // If its found send it as response
            // This used `new_raw` to send the file as raw bytes not a string
            // This may not be useful for html files but if you want to to serve an image file this will be useful
            Ok(content) => Response::new()
                .status(200)
                .bytes(content)
                .header("Content-Type", "text/html"),

            // If not send a 404 error
            Err(_) => Response::new()
                .status(404)
                .text("Not Found :/")
                .header("Content-Type", "text/html"),
        }
    });

    println!("[02] Listening on http://{}:{}", server.ip, server.port);

    // Start the server
    // This will block the current thread
    server.start().unwrap();
}
