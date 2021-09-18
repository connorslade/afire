use afire::{Header, Method, Response, Server};

// Create a new basic server like in example 01
// However, we want to use a thread pool to handle the requests
// This is incredibly simple in afire

// In production, you would probably want to use a reverse proxy like nginx
// or something similar to split the load across multiple servers
// But just a thread pool is a good way to get started

fn main() {
    // Create a new Server instance on localhost port 8080
    let mut server: Server = Server::new("localhost", 8080);

    // Define a handler for GET "/"
    server.route(Method::GET, "/", |_req| {
        Response::new(
            200,
            "Hello from ThreadPool!",
            vec![Header::new("Content-Type", "text/plain")],
        )
    });

    println!(
        "[10] Listening on http://{}:{}",
        server.ip_string(),
        server.port
    );

    // Start the server with 8 threads
    // This will block the current thread
    server.start_threaded(8);
}
