use afire::prelude::*;

struct Middle;

impl Middleware for Middle {
    fn post(&self, req: &error::Result<Request>, _res: &error::Result<Response>) -> MiddleResponse {
        if let Ok(req) = req {
            println!("{} {}", req.method, req.path)
        }

        MiddleResponse::Continue
    }
}

fn main() {
    // Create a new Server instance on localhost port 8080
    let mut server = Server::<()>::new("localhost", 8080);

    // Define a handler for GET "/"
    server.route(Method::GET, "/p/{URL}", |req| {
        Response::new()
            .text(req.path_param("URL").unwrap())
            .content(Content::TXT)
    });

    server.route(Method::POST, "/upload", |req| {
        Response::new()
            .content(Content::Custom(&req.header("Content-Type").unwrap()))
            .bytes(Vec::new())
    });

    Middle.attach(&mut server);

    // Start the server
    // This will block the current thread
    server.start().unwrap();
}
