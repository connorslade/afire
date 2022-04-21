use afire::{
    extension::cache::{to_cache, Cache},
    Method, Middleware, Response, Server,
};

fn main() {
    let mut server: Server = Server::new("localhost", 8080);

    server.route(Method::GET, "/hello/{name}", |req| {
        let name = req.path_param("name").unwrap();
        Response::new().text(format!("Hello {name}!"))
    });
    Cache::new()
        .to_cache(|x| to_cache::path_match(x, &vec!["/"]))
        .attach(&mut server);

    server.start_threaded(64).unwrap();
}
