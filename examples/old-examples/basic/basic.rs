use afire::{Content, HeaderName, Method, Response, Server, Status};

use crate::Example;

// You can run this example with `cargo run --example basic -- basic`

// In this example we will create a basic server that responds to GET requests to "/" with a simple text response
// Note: the only code here you need to worry about is the code in the exec method

pub struct Basic;

impl Example for Basic {
    fn name(&self) -> &'static str {
        "basic"
    }

    fn exec(&self) {
        // Create a new Server instance on localhost port 8080
        // The type parameter is for a server wide state, which we don't need yet so we use ()
        // In this example we are setting the ip with a string, but you can also use a Ipv4Addr or [u8; 4]
        let mut server = Server::<()>::new("localhost", 8080);

        // Define a handler for GET "/"
        server.route(Method::GET, "/", |_req| {
            Response::new()
                // By default the status is 200 (OK)
                // You can also define it yourself with the status method
                .status(Status::Ok)
                // By default the reason phrase is derived from the status
                // But you can also define it yourself with the reason method
                .reason("OK!")
                // Although is is named `text` it takes any type that implements Display
                // So for example numbers, or a serde_json::Value will work
                .text("Hi :P")
                // You can also add headers
                // The header method will take a HeaderName, String, or &str and the value can be a String or &str
                // (this is not the proper way to use the Date header, but it works for this example)
                .header(HeaderName::Date, "today")
                // Now we will set the content type to text/plain; charset=utf-8
                // The content method just adds a Content-Type header
                .content(Content::TXT)
        });

        // Start the server in single threaded mode
        // This will block the current thread
        server.start().unwrap();

        // Now navigate to http://localhost:8080 in your browser
        // You should see "Hi :P"
    }
}
