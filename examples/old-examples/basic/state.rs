use std::sync::atomic::{AtomicUsize, Ordering};

use afire::{Method, Response, Server};

use crate::Example;

// You can run this example with `cargo run --example basic -- state`

// Create a structure to hold app state
// The state is immutable, so you need to use an atomic type or a Interior Mutability type

#[derive(Default)]
struct App {
    count: AtomicUsize,
}

pub struct State;

impl Example for State {
    fn name(&self) -> &'static str {
        "state"
    }

    fn exec(&self) {
        // Create a server on localhost port 8080 with a state of App
        let mut server = Server::<App>::new("localhost", 8080).state(App::default());

        // Add catch all route that takes in state and the request
        server.stateful_route(Method::ANY, "**", |sta, _req| {
            // Respond with and increment request count
            Response::new().text(sta.count.fetch_add(1, Ordering::Relaxed))
        });

        // Start the server
        // This will block the current thread
        // Because there is a stateful route, this will panic if no state is set
        server.start().unwrap();

        // Now go to http://localhost:8080
        // You should see the request count increment each time you refresh
    }
}
