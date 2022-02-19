use afire::{Method, Response, Server};

// Use Path params to send data through a route path
// You can also add `*` segments to match with any text

fn main() {
    // Create a new Server instance on localhost port 8080
    let mut server: Server = Server::new("localhost", 8081);

    // Define a handler for GET "/greet/{name}"
    // This will handel requests with anything where the {name} is
    // This includes "/greet/bob", "/greet/fin"
    server.route(Method::GET, "/greet/{name}", |req| {
        // Get name path param
        let name = req.path_param("name").unwrap();

        // Make a nice Message to send
        let message = format!("Hello, {}", name);

        // Send Response
        Response::new()
            .text(message)
            .header("Content-Type", "text/plain")
    });

    // Define a greet route for Darren because he is very cool
    // This will take priority over the other route as it is defind after
    server.route(Method::GET, "/greet/Darren/", |_req| {
        Response::new().text("Hello, Darren. You are very cool")
    });

    println!("[13] Listening on http://{}:{}", server.ip, server.port);

    // Start the server
    // This will block the current thread
    server.start().unwrap();
}
