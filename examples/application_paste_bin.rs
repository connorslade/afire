//! A simple in memory pastebin backend
// If you want to make a real paste bin use a database for storage

// For a full pastebin front end and back end check out https://github.com/Basicprogrammer10/plaster-box
// Or try it out at https://paste.connorcode.com

use std::sync::{Arc, Mutex};
use std::time::Instant;

use afire::{Content, Method, Response, Server};

const DATA_LIMIT: usize = 1000;

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
    let mut server = Server::<()>::new("localhost", 8080);
    let pub_pastes = Arc::new(Mutex::new(Vec::new()));

    // New paste Handler
    let pastes = pub_pastes.clone();
    server.route(Method::POST, "/new", move |req| {
        // Make sure paste data isent too long
        if req.body.len() > DATA_LIMIT {
            return Response::new().status(400).text("Data too big!");
        }

        // Get the data as string
        let body_str = match req.body_string() {
            Some(i) => i,
            None => return Response::new().status(400).text("Invalid Text"),
        };

        // Get the name from the Name header
        let name = req.header("Name").unwrap_or_else(|| "Untitled".to_owned());

        let paste = Paste {
            name,
            body: body_str,
            time: Instant::now(),
        };

        // Push this paste to the pastes vector
        let mut pastes = pastes.lock().unwrap();
        let id = pastes.len();
        pastes.push(paste);

        // Send Redirect response
        Response::new()
            .status(301)
            .text("Ok")
            .header("Location", format!("/p/{}", id))
    });

    // Get pate handler
    let pastes = pub_pastes.clone();
    server.route(Method::GET, "/p/{id}", move |req| {
        // Get is from path param
        let id = req.path_param("id").unwrap().parse::<usize>().unwrap();

        // Get the paste by id
        let paste = &pastes.lock().unwrap()[id];

        // Send paste
        Response::new().text(&paste.body)
    });

    // View all pastes
    let pastes = pub_pastes.clone();
    server.route(Method::GET, "/pastes", move |_req| {
        // Starter HTML
        let mut out = String::from(
            "<meta charset=\"UTF-8\"><table><tr><th>Name</th><th>Date</th><th>Link</th></tr>",
        );

        // Add a table row for each paste
        for (i, e) in pastes.lock().unwrap().iter().enumerate() {
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
