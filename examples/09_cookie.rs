use afire::{Header, Method, Response, Server, SetCookie};

// Send data to server with a Query String and Form Data

fn main() {
    // Create a new Server instance on localhost port 8080
    let mut server: Server = Server::new("localhost", 8080);

    // Define a route to show request cookies as a table
    server.route(Method::GET, "/", |req| {
        // Return all cookies in a html table
        let mut html = String::new();
        html.push_str("<style>table, th, td {border:1px solid black;}</style>");
        html.push_str("<table>");
        html.push_str("<tr><th>Name</th><th>Value</th></tr>");
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

    // Set a cookie defined in the Query
    server.route(Method::GET, "/set", |req| {
        // Create a new cookie
        let cookie = SetCookie::new(
            &req.query.get("name").unwrap_or("test".to_string()),
            &req.query.get("value").unwrap_or("test".to_string()),
        )
        // Set some options
        .set_max_age(60 * 60)
        .set_path("/")
        .clone();

        let body = format!(
            "Set Cookie '{}' to '{}'",
            cookie.cookie.name, cookie.cookie.value
        );

        // Set the cookie
        Response::new(200, &body, vec![Header::new("Content-Type", "text/html")])
            .add_cookie(cookie)
            .clone()
    });

    // Now goto http://localhost:8080/set?name=hello&value=world
    // Then goto http://localhost:8080/ and you should see a table with the cookie

    println!(
        "[09] Listening on http://{}:{}",
        server.ip_string(),
        server.port
    );

    // Start the server
    // This will block the current thread
    server.start();
}
