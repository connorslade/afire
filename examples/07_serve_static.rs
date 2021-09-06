use afire::{Header, Response, Server};
use std::fs;

// Serve static files from a directory

const STATIC_DIR: &str = "examples/data";

fn main() {
    // Create a new Server instance on localhost port 8080
    let mut server: Server = Server::new("localhost", 8080);

    // Define a method to handle all requests
    // Other methods can be defined after this one and take a higher priority
    server.all(|req| {
        // Gen the local path to the requested file
        // Im removing '/..' in the path to avoid directory traversal exploits
        let mut path = format!("{}{}", STATIC_DIR, req.path.replace("/..", ""));

        // Add Index.html if path ends with /
        // This will cause the server to automatically serve the index.html
        if path.ends_with('/') {
            path.push_str("index.html");
        }

        // Also add '/index.html' if path dose not end with a file
        // Ex 'page' will return 'page/index.html'
        if !path.split('/').last().unwrap_or_default().contains('.') {
            path.push_str("/index.html");
        }

        // Try to read File
        // Using read over read_to_string is important to allow serving non utf8 files
        match fs::read(&path) {
            // If its found send it as response
            // We are setting the Content-Type header with the file extension through a match expression
            Ok(content) => Response::new_raw(
                200,
                content,
                vec![Header::new("Content-Type", get_type(&path))],
            ),

            // If not read and send 404.html
            // If that file is not found, fallback to sending "Not Found :/"
            Err(_) => Response::new_raw(
                404,
                fs::read(format!("{}/404.html", STATIC_DIR))
                    .unwrap_or_else(|_| "Not Found :/".as_bytes().to_owned()),
                vec![Header::new("Content-Type", "text/html")],
            ),
        }
    });

    println!(
        "[07] Listening on http://{}:{}",
        server.ip_string(),
        server.port
    );

    // Start the server
    // This will block the current thread
    server.start();
}

// Get the type MMIE content type of a file from its extension
// Thare are lots of other MMIME types but these are the most common
fn get_type(path: &str) -> &str {
    match path.split('.').last() {
        Some(ext) => match ext {
            "html" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "png" => "image/png",
            "jpg" => "image/jpeg",
            "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "ico" => "image/x-icon",
            "svg" => "image/svg+xml",
            _ => "application/octet-stream",
        },

        None => "application/octet-stream",
    }
}
