use afire::{
    extensions::logger::{Level, Logger},
    Content, HeaderName, Method, Middleware, Response, Server,
};

use crate::Example;

// You can run this example with `cargo run --example basic -- logging`

// Use some of afire's built-in middleware to log requests.

pub struct Logging;

impl Example for Logging {
    fn name(&self) -> &'static str {
        "logging"
    }

    fn exec(&self) {
        // Create a new Server instance on localhost port 8080
        let mut server = Server::<()>::new("localhost", 8080);

        // Define a handler for GET "/"
        server.route(Method::GET, "/", |_req| {
            Response::new()
                .text("Hello World!\nThis request has been logged!")
                .content(Content::TXT)
        });

        // Make a logger and attach it to the server

        // By default Log Level is INFO, File is None and Console is true
        // This could be condensed to `Logger::new().attach(&mut server);` as it uses al default values
        Logger::new()
            // The level of logging this can be Debug or Info
            // Debug will give a lot more information about the request
            .level(Level::Info)
            // This will have Logger make use of the RealIp extention,
            // which will allow logging the correct IP when using a reverse proxy.
            .real_ip(HeaderName::XForwardedFor)
            // The file argument tells the logger if it should save to a file
            // Only one file can be defined per logger
            // With logging to file it will write to the file on every request... (for now)
            .file("example.log")
            .unwrap()
            // Tells the Logger it should log to the console as well
            .console(true)
            // This must be put at the end of your Logger Construction
            // It adds the Logger to your Server as Middleware
            .attach(&mut server);

        // Now if you goto http://localhost:8080/ you should see the log message in console.

        // Start the server
        // This will block the current thread
        server.start().unwrap();
    }
}
