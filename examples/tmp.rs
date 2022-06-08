use afire::prelude::*;

fn main() {
    // Create a new Server instance on localhost port 8080
    let mut server = Server::<()>::new("localhost", 8080);

    // Define a handler for GET "/"
    server.route(Method::GET, "/", |req| {
        Response::new().text(req.body_string().unwrap())
    });

    // Start the server
    // This will block the current thread
    server.start().unwrap();
}
