//! A simple in memory pastebin backend
// If you want to make a real paste bin use a database for storage

// For a full pastebin front end and back end check out https://github.com/Basicprogrammer10/plaster-box
// Or try it out at https://paste.connorcode.com

use std::str::FromStr;
use std::time::Instant;
use std::{borrow::Borrow, sync::RwLock};

use afire::internal::encoding::decode_url;
use afire::{Content, HeaderType, Method, Query, Response, Server, Status};

const DATA_LIMIT: usize = 10_000;

const TIME_UNITS: &[(&str, u16)] = &[
    ("second", 60),
    ("minute", 60),
    ("hour", 24),
    ("day", 30),
    ("month", 12),
    ("year", 0),
];

struct Paste {
    name: String,
    body: String,
    time: Instant,
}

fn main() {
    // Create Server
    let mut server = Server::new("localhost", 8080).state(RwLock::new(Vec::new()));

    // New paste interface
    server.route(Method::GET, "/", |_req| {
        Response::new().content(Content::HTML).text(
            r#"
        <form action="/new-form" method="post">
        <input type="text" name="name" id="name" placeholder="Title">
        
        <br />
        <textarea id="body" name="body" rows="5" cols="33"></textarea>
        <input type="submit" value="Submit" />
    </form>
    "#,
        )
    });

    // New paste API handler
    server.stateful_route(Method::POST, "/new", move |app, req| {
        // Make sure paste data isent too long
        if req.body.len() > DATA_LIMIT {
            return Response::new()
                .status(Status::NotFound)
                .text("Data too big!");
        }

        // Get the data as string
        let body_str = String::from_utf8_lossy(&req.body).to_string();

        // Get the name from the Name header
        let name = req.headers.get("Name").unwrap_or("Untitled");

        let paste = Paste {
            name: name.to_owned(),
            body: body_str,
            time: Instant::now(),
        };

        // Push this paste to the pastes vector
        let mut pastes = app.write().unwrap();
        let id = pastes.len();
        pastes.push(paste);

        // Send Redirect response
        Response::new()
            .status(Status::MovedPermanently)
            .text("Ok")
            .header(HeaderType::Location, format!("/p/{}", id))
    });

    // New paste form handler
    server.stateful_route(Method::POST, "/new-form", |app, req| {
        // Get data from response
        let query = Query::from_str(String::from_utf8_lossy(&req.body).borrow()).unwrap();
        let name = decode_url(query.get("name").unwrap_or("Untitled")).expect("Invalid name");
        let body = decode_url(query.get("body").expect("No body supplied")).expect("Invalid body");

        // Make sure paste data isent too long
        if body.len() > DATA_LIMIT {
            return Response::new()
                .status(Status::NotFound)
                .text("Data too big!");
        }

        let paste = Paste {
            name,
            body,
            time: Instant::now(),
        };

        // Push this paste to the pastes vector
        let mut pastes = app.write().unwrap();
        let id = pastes.len();
        pastes.push(paste);

        // Send Redirect response
        Response::new()
            .status(Status::MovedPermanently)
            .text("Ok")
            .header(HeaderType::Location, format!("/p/{}", id))
    });

    // Get pate handler
    server.stateful_route(Method::GET, "/p/{id}", move |app, req| {
        // Get is from path param
        let id = req.param("id").unwrap().parse::<usize>().unwrap();

        // Get the paste by id
        let paste = &app.read().unwrap()[id];

        // Send paste
        Response::new().text(&paste.body)
    });

    // View all pastes
    server.stateful_route(Method::GET, "/pastes", move |app, _req| {
        // Starter HTML
        let mut out = String::from(
            r#"<a href="/">New Paste</a><meta charset="UTF-8"><table><tr><th>Name</th><th>Date</th><th>Link</th></tr>"#,
        );

        // Add a table row for each paste
        for (i, e) in app.read().unwrap().iter().enumerate() {
            out.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td><a href=\"/p/{}\">🔗</a></td></tr>",
                e.name,
                best_time(e.time.elapsed().as_secs()),
                i
            ));
        }

        // Send HTML
        Response::new()
            .text(format!("{}</table>", out))
            .content(Content::HTML)
    });

    server.start().unwrap();
}

// Turn seconds ago into a more readable relative time
// Ex 1 minute ago or 3 years ago
pub fn best_time(secs: u64) -> String {
    let mut secs = secs as f64;

    for i in TIME_UNITS {
        if i.1 == 0 || secs < i.1 as f64 {
            secs = secs.round();
            return format!("{} {}{} ago", secs, i.0, if secs > 1.0 { "s" } else { "" });
        }

        secs /= i.1 as f64;
    }

    format!("{} years ago", secs.round())
}

// To use POST to /new with the body set to your paste data
// You can then GET /pastes to see all the pastes
