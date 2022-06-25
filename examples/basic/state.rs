use std::sync::atomic::{AtomicUsize, Ordering};

use afire::{Method, Response, Server};

use crate::Example;

// Create a structure to hold app state
// This is immutable

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
        let mut server = Server::<App>::new("localhost", 8080).state(App::default());

        // Add catch all route that takes in state and the request
        server.stateful_route(Method::ANY, "**", |sta, _req| {
            // Respond with and increment request count
            Response::new().text(sta.count.fetch_add(1, Ordering::Relaxed))
        });

        // Start the server
        // This will block the current thread
        server.start().unwrap();
    }
}
