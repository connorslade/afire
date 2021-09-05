use afire::*;
use std::fs;

const ITER: u32 = 1_000;
const BENCH: bool = false;

fn main() {
    bencher("Create Server", ITER, || {
        let _ = Server::new("127.0.0.1", 8080);
    });

    bencher("Add Rate limiter", ITER, || {
        let mut server = Server::new("127.0.0.1", 8080);
        RateLimiter::attach(&mut server, 10, 10);
    });

    bencher("Add Logger", ITER, || {
        let mut server = Server::new("127.0.0.1", 8080);
        Logger::attach(
            &mut server,
            Logger::new(Level::Debug, Some("nose.txt"), true),
        );
    });

    bencher("Add Simple Route", ITER, || {
        let mut server = Server::new("127.0.0.1", 8080);
        server.route(Method::GET, "/", |_| {
            Response::new(
                200,
                "Hello World",
                vec![Header::new("Content-Type", "text/plain")],
            )
        });
    });

    bencher("Add Middleware", ITER, || {
        let mut server = Server::new("127.0.0.1", 8080);
        server.every(Box::new(|_| None));
    });

    let mut server: Server = Server::new("localhost", 1234);

    // Enable Rate Limiting
    RateLimiter::attach(&mut server, 10, 10);

    // Enable Logging
    Logger::attach(
        &mut server,
        Logger::new(Level::Debug, Some("nose.txt"), true),
    );

    // Define a handler for GET "/"
    server.route(Method::GET, "/", |_req| {
        Response::new(
            200,
            "Hi :P",
            vec![Header::new("Content-Type", "text/plain")],
        )
    });

    // Define a handler for GET "/nose"
    server.route(Method::GET, "/nose", |_req| {
        Response::new(
            200,
            "N O S E",
            vec![Header::new("Content-Type", "text/plain")],
        )
    });

    // Define a handler for ANY "/hi"
    server.route(Method::GET, "/hi", |_req| {
        Response::new(
            200,
            "<h1>Hello, How are you?</h1>",
            vec![Header::new("Content-Type", "text/html")],
        )
    });

    // Serve a file
    server.route(Method::GET, "/pi", |_req| {
        Response::new(
            200,
            &fs::read_to_string("data/index.txt").unwrap(),
            vec![Header::new("Content-Type", "text/html")],
        )
    });

    // Redirecting to a different URL
    server.route(Method::GET, "/connorcode", |_req| {
        Response::new(
            // Needs a status of 301, 302, 303, 307, 308 to redirect
            301,
            // Data is not really important
            "Hello, Connor",
            vec![
                Header::new("Content-Type", "text/plain"),
                // Needs a Location header
                // This can be a relative URL or an absolute URL
                Header::new("Location", "https://connorcode.com"),
            ],
        )
    });

    // Serve an image
    server.route(Method::GET, "/favicon.ico", |_req| {
        let bytes = &fs::read("data/favicon.ico").unwrap();

        Response::new_raw(
            200,
            bytes.to_vec(),
            vec![Header::new("Content-Type", "image/x-icon")],
        )
    });

    // Start the server
    server.start();
}

fn bencher(name: &str, iter: u32, f: fn() -> ()) {
    if !BENCH {
        return;
    }

    let mut avg_time: u128 = 0;
    for _ in 0..iter {
        let start = std::time::Instant::now();
        f();
        avg_time += start.elapsed().as_nanos();
    }
    println!("[*] Bench: {} ({})", name, iter);
    println!(" ├─ Total: {}ns", avg_time);
    println!(" └─ AVG: {}ns\n", avg_time / iter as u128);
}
