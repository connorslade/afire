//! A simple in memory pastebin
use std::sync::{Arc, Mutex};
use std::time::Instant;

use afire::{Method, Response, Server};

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
    let mut server = Server::new("localhost", 8080);
    let pub_pastes = Arc::new(Mutex::new(Vec::new()));

    // New paste Handler
    let pastes = pub_pastes.clone();
    server.route(Method::POST, "/new", move |req| {
        if req.body.len() > DATA_LIMIT {
            return Response::new().status(400).text("Data too big!");
        }

        let body_str = match req.body_string() {
            Some(i) => i,
            None => return Response::new().status(400).text("Invalid Text"),
        };

        let name = req.header("Name").unwrap_or_else(|| "Untitled".to_owned());

        let paste = Paste {
            name,
            body: body_str,
            time: Instant::now(),
        };

        let mut pastes = pastes.lock().unwrap();
        let id = pastes.len();
        pastes.push(paste);

        Response::new()
            .status(301)
            .text("Ok")
            .header("Location", format!("/p/{}", id))
    });

    // Get pate handler
    let pastes = pub_pastes.clone();
    server.route(Method::GET, "/p/{id}", move |req| {
        let id = req.path_param("id").unwrap().parse::<usize>().unwrap();
        let paste = &pastes.lock().unwrap()[id];

        Response::new().text(&paste.body)
    });

    // View all pastes
    let pastes = pub_pastes.clone();
    server.route(Method::GET, "/pastes", move |_req| {
        let mut out = String::from(
            "<meta charset=\"UTF-8\"><table><tr><th>Name</th><th>Date</th><th>Link</th></tr>",
        );
        for (i, e) in pastes.lock().unwrap().iter().enumerate() {
            out.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td><a href=\"/p/{}\">🔗</a></td></tr>",
                e.name,
                best_time(e.time.elapsed().as_secs()),
                i
            ));
        }

        Response::new().text(out)
    });

    server.start().unwrap();
}

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
