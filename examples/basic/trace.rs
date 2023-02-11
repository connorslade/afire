use afire::{
    trace,
    trace::{set_log_level, Level},
    Method, Response, Server,
};

use crate::Example;

// You can run this example with `cargo run --example basic -- trace`

// In this example we will learn about afire's built-in tracing system.
// Because of how its used internally there is a global log level, not one per server.
// You can set this level with `afire::trace::set_log_level`, which takes one parameter a `afire::trace::Level`.
// You can also set wether the logs have ANSI codes for color using `afire::trace::set_log_color`, color is enabled by default.
// Now to use the logger there is the `trace!` macro, which you can use one of two different ways:
//
// trace!(Level::<LOG_LEVEL>, <FORMATTED ARGS>)
// trace!(<FORMATTED ARGS>) // uses the Trace level
//
// // Examples
// let a = 100;
// trace!("The var a is currently {a}");
// trace!(Level::Error, "An error occurred!");
//
// The Log Levels are in the following order, with the more verbose levels at the bottom.
// Setting the log level to Off will disable all logging.
// Also note that Error is the default level.
//
// - Off
// - Error
// - Trace
// - Debug

pub struct Trace;

impl Example for Trace {
    fn name(&self) -> &'static str {
        "trace"
    }

    fn exec(&self) {
        // Set the log level to Trace (shows some helpful information during startup)
        // The default is Level::Error
        set_log_level(Level::Trace);
        trace!(Level::Trace, "Setting log level to Trace");
        trace!(Level::Error, "Example error message");

        // Create a new Server instance on localhost port 8080
        let mut server = Server::<()>::new("localhost", 8080);

        server.route(Method::GET, "/", |req| {
            // The default log level is Level::Trace so this will be logged
            trace!("Request from {}", req.address.ip());
            Response::new()
        });

        server.start().unwrap();
    }
}
