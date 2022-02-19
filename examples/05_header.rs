use afire::{Header, Method, Response, Server};

// Read request headers and send a response with custom headers
// You may want to read https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers to understand headers more

fn main() {
    // Create a new Server instance on localhost port 8080
    let mut server: Server = Server::new("localhost", 8080)
        // Define server wide default headers
        // These will be send with every response
        // If the same header is defined in the route it will be put before the default header
        // Although it is not garunteed to be the one picked by the client it usually is
        // At the bottom of this file is a representation of the order of the headers
        .default_header("X-Server-Header", "This is a server wide header");

    // Define a route to redirect to another website
    server.route(Method::GET, "/", |_req| {
        // Because this is all bout headers I have put the header vector here
        let headers = vec![
            // Tell the client what type of data we are sending
            Header::new("Content-Type", "text/html"),
            // Tell the client to redirect to another website
            Header::new("Location", "https://connorcode.com"),
            // Custom header
            Header::new("X-Custom-Header", "This is a custom header"),
        ];

        // Define response body
        // In this case this should only be seen if the client doesn't support redirects for some reason
        let text = "<a href=\"https://connorcode.com\">connorcode</a>";

        // The response code of 308 tells the client to redirect to the location specified in the header
        // Thare are other response codes you can use too
        // 301 -> Moved Permanently
        // 302 -> Found
        // 303 -> See Other
        // 307 -> Temporary Redirect
        // 308 -> Permanent Redirect
        Response::new().status(308).text(text).headers(headers)
    });

    // Now to define a route to handle client headers
    // This will just echo the headers back to the client
    server.route(Method::GET, "/headers", |req| {
        // Get the headers from the request and make a html string
        let body = req
            .headers
            .iter()
            .fold(String::new(), |old, new| old + &format!("{:?}<br />", new));

        // Create a response with the headers
        Response::new()
            .status(200)
            .text(body)
            .header("Content-Type", "text/html")
    });

    // You can now goto http://localhost:8080 you should see a redirect to https://connorcode.com
    // And you can goto http://localhost:8080/headers to see the headers your client sent to the server

    println!("[05] Listening on http://{}:{}", server.ip, server.port);

    // Start the server
    // This will block the current thread
    server.start().unwrap();
}

// This is a representation of the order of the headers
// The order is important because the client usually picks the first header it can find

// -------------------
// |  Route Headers  |
// -------------------
// | Default Headers |
// -------------------
// |  ContentLength  |
// -------------------
