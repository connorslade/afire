use afire::*;

fn main() {
    let mut s = Server::new("localhost", 9090);

    s.route(Method::GET, "/", |_req| {
        Response::new().text("Hello").content(Content::HTML)
    });

    s.start().unwrap();
}
