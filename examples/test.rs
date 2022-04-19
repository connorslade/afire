use std::sync::atomic::{AtomicUsize, Ordering};

use afire::{
    extension::{Level, Logger},
    Method, Middleware, Response, Server,
};

#[derive(Default)]
struct App {
    count: AtomicUsize,
}

fn main() {
    let mut server = Server::<App>::new("localhost", 8080).state(App::default());

    // server.route(Method::GET, "/sl", |_req| Response::new().text("wllo"));
    server.stateful_route(Method::GET, "**", |sta, _req| {
        sta.count.fetch_add(1, Ordering::Relaxed);
        Response::new().text(sta.count.load(Ordering::Relaxed).to_string())
    });
    Logger::new().level(Level::Debug).attach(&mut server);

    server.start_threaded(64).unwrap();
}
