use afire::{Header, Method, Query, Response, Server};

// Send data to server with a Query String, Path params and Form Data

fn main() {
    // Create a new Server instance on localhost port 8080
    let mut server: Server = Server::new("localhost", 8080);

    // Define a route to handel query string
    // This will try to find a name value pair in the query string
    server.route(Method::GET, "/", |req| {
        // Format the response text
        let text = format!(
            "<h1>Hello, {}!</h1>",
            // Get the query value of name and default to "Nobody" if not found
            req.query
                .get("name")
                .unwrap_or_else(|| "Nobody".to_string())
        );

        Response::new()
            .status(200)
            .text(text)
            .header(Header::new("Content-Type", "text/html"))
    });

    // Define another route
    // This time to handle form data
    server.route(Method::POST, "/form", |req| {
        // The body of requests is not part of the req.query
        // Instead it is part of the req.body but as a string
        // We will need to parse it get it as a query
        // This is *super* easy to do with afire
        let body_data = Query::from_body(String::from_utf8(req.body).unwrap()).unwrap();

        let name = body_data
            .get("name")
            .unwrap_or_else(|| "Nobody".to_string());
        let text = format!("<h1>Hello, {}</h1>", name);

        Response::new()
            .status(200)
            .text(text)
            .header(Header::new("Content-Type", "text/html"))
    });

    // Define webpage with form
    // The form data will be post to /form on submit
    server.route(Method::GET, "/form", |_req| {
        let page = r#"<form method="post">
            <label for="name">Name:</label>
            <input type="text" id="name" name="name"><br><br>
            <input type="submit" value="Submit">
      </form>"#;

        Response::new()
            .status(200)
            .text(page)
            .header(Header::new("Content-Type", "text/html"))
    });

    // Define a page with path params
    // As this is not built into afire you will need to use middleware (for now :P)
    // This is like "/greet/:name"
    server.middleware(Box::new(|req| {
        // Move on if path dosent start with /path or the method is not get
        // You could also do this with a regex (but afire is dependency free so I wont show that in this example)
        if !req.path.starts_with("/greet/") || req.method != Method::GET {
            return None;
        }

        // Extract the param value
        // This is getting the 2rd path value
        // Ex: /greet/Darren/
        let name = req.path.split("/").nth(2).unwrap();

        // Respond with "Hello, {{NAME}}"
        Some(
            Response::new()
                .text(format!("<h1>Hello, {}</h1>", name))
                .header(Header::new("Content-Type", "text/html")),
        )
    }));

    // You can now goto http://localhost:8080?name=John and should see "Hello, John"
    // If you goto http://localhost:8080/form and submit the form you should see "Hello, {NAME}"
    // Also goto http://localhost:8080/greet/John and you should see "Hello, John"

    println!(
        "[04] Listening on http://{}:{}",
        server.ip_string(),
        server.port
    );

    // Start the server
    // This will block the current thread
    server.start().unwrap();
}
