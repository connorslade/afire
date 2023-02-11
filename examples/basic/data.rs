use std::net::Ipv4Addr;

use afire::{Content, HeaderType, Method, Query, Response, Server};

use crate::Example;

// You can run this example with `cargo run --example basic -- data`

// In this example we will work with data in the request using query params and form data

pub struct Data;

impl Example for Data {
    fn name(&self) -> &'static str {
        "data"
    }

    fn exec(&self) {
        // Create a new Server instance on localhost port 8080
        let mut server = Server::<()>::new(Ipv4Addr::LOCALHOST, 8080);

        // Define a route to handel query string
        // This will try to find a name value pair in the query string
        server.route(Method::GET, "/", |req| {
            // Format the response text
            let text = format!(
                "<h1>Hello, {}!</h1>",
                // Get the query value of name and default to "Nobody" if not found
                req.query.get("name").unwrap_or("Nobody")
            );

            Response::new().text(text).content(Content::HTML)
        });

        // Define another route
        // This time to handle form data
        server.route(Method::POST, "/form", |req| {
            // The body of requests is not part of the req.query
            // Instead it is part of the req.body but as a string
            // We will need to parse it get it as a query
            let body_data = Query::from_body(&String::from_utf8_lossy(&req.body));

            let name = body_data.get("name").unwrap_or("Nobody");
            let text = format!("<h1>Hello, {}</h1>", name);

            // Create a new response, with the following default data
            // - Status: 200
            // - Data: OK
            // - Headers: []
            Response::new()
                // Set the response body to be text
                .text(text)
                // Set the `Content-Type` header to be `text/html`
                // Note: This could also be set with the Response::content method
                .header(HeaderType::ContentType, "text/html")
        });

        // Define webpage with form
        // The form data will be post to /form on submit
        server.route(Method::GET, "/form", |_req| {
            let page = r#"<form method="post">
            <label for="name">Name:</label>
            <input type="text" id="name" name="name"><br><br>
            <input type="submit" value="Submit">
      </form>"#;

            Response::new().text(page).content(Content::HTML)
        });

        // Define a page with path params
        server.route(Method::GET, "/greet/{name}", |req| {
            // As this route would ever run without all the path params being filled
            // It is safe to unwrap if the name is in the path
            let data = format!("<h1>Hello, {}</h1>", req.param("name").unwrap());

            Response::new().text(data).content(Content::HTML)
        });

        // You can now goto http://localhost:8080?name=John and should see "Hello, John"
        // If you goto http://localhost:8080/form and submit the form you should see "Hello, {NAME}"
        // Also goto http://localhost:8080/greet/John and you should see "Hello, John"

        // Start the server
        // This will block the current thread
        server.start().unwrap();
    }
}
