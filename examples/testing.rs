use afire::{Header, Method, Query, Response, Server};

// Send data to server with a Query String and Form Data

fn main() {
    // Create a new Server instance on localhost port 8080
    let mut server: Server = Server::new("localhost", 9191);

    // Define a route to handel query string
    // This will try to find a name value pair in the query string
    server.route(Method::GET, "/", |req| {
        // Return all cookies in a html table
        let mut html = String::new();
        html.push_str("<style>table, th, td {border:1px solid black;}</style>");
        html.push_str("<table>");
        for cookie in req.cookies {
            html.push_str("<tr><td>");
            html.push_str(&cookie.name);
            html.push_str("</td><td>");
            html.push_str(&cookie.value);
            html.push_str("</td></tr>");
        }
        html.push_str("</table>");

        Response::new(200, &html, vec![Header::new("Content-Type", "text/html")])
    });

    // Start the server
    // This will block the current thread
    server.start();
}
