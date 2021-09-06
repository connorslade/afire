use afire::{Header, Method, Response, Server};

// Define middleware that will be executed before the request handler.
// In afire middleware are run before the normal routes and they can ether
// send a response or pass the request to the next middleware / route.

fn main() {
    // Create a new Server instance on localhost port 8080
    let mut server: Server = Server::new("localhost", 8080);

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
        Response::new(308, text, headers)
    });

    // Now to define a route to handle client headers
    // This will just echo the headers back to the client
    server.route(Method::GET, "/headers", |req| {
        let mut body = "".to_string();

        // Get the headers from the request and make a html string
        for i in req.headers {
            body += &format!("{:?}<br />", i);
        }

        // Create a response with the headers
        Response::new(200, &body, vec![Header::new("Content-Type", "text/html")])
    });

    // Define server wide default headers
    // These will be send with every response
    // Default headers also have a higher priority than route specific headers so they will override them
    server.add_default_header(Header::new(
        "X-Server-Header",
        "This is a server wide header",
    ));

    // You can now goto http://localhost:8080 you should see a redirect to https://connorcode.com
    // And you can goto http://localhost:8080/headers to see the headers your client sent to the server

    println!(
        "[08] Listening on http://{}:{}",
        server.ip_string(),
        server.port
    );

    // Start the server
    // This will block the current thread
    server.start();
}
