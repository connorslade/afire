use afire::{
    extension::cache::{to_cache, Cache},
    Method, Middleware, Response, Server,
};

fn main() {
    let mut server: Server = Server::new("localhost", 8080);

    server.route(Method::GET, "/", |req| {
        std::thread::sleep_ms(10000);
        Response::new().text("OK")
    });
    Cache::new()
        .to_cache(|x| to_cache::path_match(x, &vec!["/"]))
        .attach(&mut server);

    server.start().unwrap();
}
