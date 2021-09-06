use afire::{Header, Method, Response, Server};
use std::fs;

// Serve a local file
// On each request, the server will read the file and send it to the client.

fn main() {
    // Create a new Server instance on localhost port 8080
    let mut server: Server = Server::new("localhost", 8080);

    // Define a handler for GET "/"
    server.route(Method::GET, "/", |_req| {
        // Try to read File
        match fs::read("data/02/index.html") {
            // If its found send it as response
            // This used `new_raw` to send the file as raw bytes not a string
            // This may not be useful for html files but if you want to to serve an image file this will be useful
            Ok(content) => {
                Response::new_raw(200, content, vec![Header::new("Content-Type", "text/html")])
            }

            // If not send 404.html
            Err(_) => Response::new(
                404,
                "Not Found :/",
                vec![Header::new("Content-Type", "text/html")],
            ),
        }
    });

    println!(
        "[02] Listening on http://{}:{}",
        server.ip_string(),
        server.port
    );

    // Start the server
    // This will block the current thread
    server.start();
}
