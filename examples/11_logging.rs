use afire::{Header, Level, Logger, Method, Response, Server};

// Use some of afire's built-in middleware to log requests.

fn main() {
    // Create a new Server instance on localhost port 8080
    let mut server: Server = Server::new("localhost", 8080);

    // Define a handler for GET "/"
    server.route(Method::GET, "/", |_req| {
        Response::new(
            200,
            "Hello World!\nThis request has been logged!",
            vec![Header::new("Content-Type", "text/plain")],
        )
    });

    // Make a logger
    // This can be simplified by putting the logger creation directly in the attach method

    // The fist argument is the level of logging this can be Debug or Info
    // Debug will give alot more information about the request

    // The second argument is if the logger should save to a file or not
    // None is no file and Some(String) is a file with the given name

    // The third argument tells the logger should print to the console or not
    let logger = Logger::new(Level::Info, None, true);

    // Attach a logger to the server
    Logger::attach(&mut server, logger);

    // Now if you goto http://localhost:8080/ you should see the log message in console.

    println!(
        "[11] Listening on http://{}:{}",
        server.ip_string(),
        server.port
    );

    // Start the server
    // This will block the current thread
    server.start();
}
