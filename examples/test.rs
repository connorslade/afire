use std::sync::atomic::{AtomicUsize, Ordering};

use afire::{extension::RequestId, Method, Middleware, Response, Server};

#[derive(Default)]
struct App {
    count: AtomicUsize,
}

fn main() {
    let mut server = Server::<App>::new("localhost", 8080); //.state();

    server.route(Method::GET, "/sl", |req| Response::new().text("wllo"));
    server.stateful_route(Method::GET, "/", |sta, req| {
        sta.count.fetch_add(1, Ordering::Relaxed);
        Response::new().text(sta.count.load(Ordering::Relaxed).to_string())
    });
    RequestId::new("X-REQ-ID").attach(&mut server);

    server.start().unwrap();
}
