use afire::{Content, Method, Response, Server, SetCookie};

use crate::Example;

// You can run this example with `cargo run --example basic -- cookie`

// This example shows how you can work with cookies using the Cookie and SetCookie structs
// We will make a route to show all cookies and a route to set a cookie

pub struct Cookie;

impl Example for Cookie {
    fn name(&self) -> &'static str {
        "cookie"
    }

    fn exec(&self) {
        // Create a new Server instance on localhost port 8080
        let mut server = Server::<()>::new([127, 0, 0, 1], 8080);

        // Define a route to show request cookies as a table
        server.route(Method::GET, "/", |req| {
            // Return all cookies in a *messy* html table
            let mut html = String::new();
            html.push_str("<style>table, th, td {border:1px solid black;}</style>");
            html.push_str("<table>");
            html.push_str("<tr><th>Name</th><th>Value</th></tr>");
            for cookie in &*req.cookies {
                html.push_str("<tr><td>");
                html.push_str(&cookie.name);
                html.push_str("</td><td>");
                html.push_str(&cookie.value);
                html.push_str("</td></tr>");
            }
            html.push_str("</table>");

            Response::new().text(html).content(Content::HTML)
        });

        // Set a cookie defined in the Query
        server.route(Method::GET, "/set", |req| {
            // Create a new cookie
            let cookie = SetCookie::new(
                req.query.get("name").unwrap_or("test"),
                req.query.get("value").unwrap_or("test"),
            )
            // Set some options
            .max_age(60 * 60)
            .path("/");

            let body = format!(
                "Set Cookie '{}' to '{}'",
                cookie.cookie.name, cookie.cookie.value
            );

            // Set the cookie
            Response::new()
                .text(body)
                .content(Content::HTML)
                .cookie(cookie)
        });

        // Now goto http://localhost:8080/set?name=hello&value=world
        // Then goto http://localhost:8080/ and you should see a table with the cookie

        // Start the server in single threaded mode
        // This will block the current thread
        server.start().unwrap();
    }
}
