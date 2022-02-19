use afire::{Response, ServeStatic, Server};

// Serve static files from a directory
// Afire middleware makes this *easy*

const STATIC_DIR: &str = "examples/data";

fn main() {
    // Create a new Server instance on localhost port 8080
    let mut server: Server = Server::new("localhost", 8080);

    // Make a new static file server with a path
    ServeStatic::new(STATIC_DIR)
        // Middleware here works much diffrently to afire middleware
        // The middleware priority is still by most recently defined
        // But this middleware takes functions only - no closures
        // and resultes of the middleware are put togther so more then one ac affect thre response
        //
        // Args:
        // - req: Client Request
        // - res: Current Server Response
        // - suc: File to serve was found
        .middleware(|req, res, suc| {
            // Print path sevred
            println!("Staticly Served: {}", req.path);

            // Return none to not mess with response
            // Or in this case add a header and pass through the sucess value
            Some((res.header("X-Static-Serve", "true"), suc))
        })
        // Function that runs when no file is found to serve
        // This will run before middleware
        .not_found(|_req, _dis| Response::new().status(404).text("Page Not Found!"))
        // Add an extra mime type to the server
        // It has alot already
        .mime_type("key", "value")
        // Attatch the middleware to the server
        .attach(&mut server);

    println!("[07] Listening on http://{}:{}", server.ip, server.port);

    // Start the server
    // This will block the current thread
    server.start().unwrap();
}
