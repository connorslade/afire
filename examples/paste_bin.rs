//! A simple in memory pastebin backend.
//! If you want to make a real paste bin you will need some sort of database for persistent storage.
//! For a full pastebin front end and back end check out https://github.com/Basicprogrammer10/plaster-box.
//! Or try it out at https://paste.connorcode.com.
//!
//! To use this example, POST to /new with the body set to your paste data or use the form at /.
//! You can then GET /pastes to see all the pastes and GET /p/{id} to see a paste.

use std::{error, sync::RwLock, time::Instant};

use afire::{
    internal::{encoding::url, sync::ForceLockRwLock},
    route::RouteContext,
    trace::{set_log_level, Level},
    Content, Error, HeaderType, Method, Query, Server, Status,
};

// Maximum size of a paste in bytes
const DATA_LIMIT: usize = 10_000;

struct Paste {
    name: String,
    body: String,
    time: Instant,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    set_log_level(Level::Trace);

    // Create a server on localhost port 8080.
    let mut server = Server::new("localhost", 8080).state(RwLock::new(Vec::new()));

    // New paste interface
    server.route(Method::GET, "/", |ctx| {
        Ok(ctx.content(Content::HTML).text(HTML).send()?)
    });

    // New paste API handler
    server.route(Method::POST, "/new", move |ctx| {
        // Make sure paste data isn't too long
        if ctx.req.body.len() > DATA_LIMIT {
            Error::bail("Data too big!")?;
        }

        // Get the data as string
        let body_str = String::from_utf8_lossy(&ctx.req.body);

        // Get the name from the Name header
        let name = ctx.req.headers.get("Name").unwrap_or("Untitled");

        let paste = Paste {
            name: name.to_owned(),
            body: body_str.to_string(),
            time: Instant::now(),
        };

        // Push this paste to the pastes vector
        let app = ctx.app();
        let mut pastes = app.write().unwrap();
        let id = pastes.len();
        pastes.push(paste);
        println!("New paste: #{}", id);

        // Send Redirect response
        ctx.status(Status::SeeOther)
            .header(HeaderType::Location, format!("/p/{id}"))
            .text(format!("Redirecting to /p/{id}."))
            .send()?;
        Ok(())
    });

    // New paste form handler
    server.route(Method::POST, "/new-form", |ctx| {
        // Get data from form
        let query = Query::from_body(&String::from_utf8_lossy(&ctx.req.body));
        let name = url::decode(query.get("name").unwrap_or("Untitled")).context("Invalid name")?;
        let body =
            url::decode(query.get("body").context("No body supplied")?).context("Invalid body")?;

        // Make sure paste data isn't too long
        if body.len() > DATA_LIMIT {
            Error::bail("Data too big!")?;
        }

        let paste = Paste {
            name,
            body,
            time: Instant::now(),
        };

        // Push this paste to the pastes vector
        let app = ctx.app();
        let mut pastes = app.force_write();
        let id = pastes.len();
        pastes.push(paste);
        println!("New paste: #{}", id);

        // Send Redirect response
        ctx.status(Status::SeeOther)
            .header(HeaderType::Location, format!("/p/{}", id))
            .text("Ok")
            .send()?;
        Ok(())
    });

    // Get pate handler
    server.route(Method::GET, "/p/{id}", move |ctx| {
        // Get is from path param
        let id = ctx.param("id").parse::<usize>().unwrap();

        // Get the paste by id
        let app = ctx.app();
        let paste = &app.read().unwrap()[id];

        // Send paste
        ctx.text(&paste.body).send()?;
        Ok(())
    });

    // View all pastes
    server.route(Method::GET, "/pastes", move |ctx| {
        // Starter HTML
        let mut out = String::from(
            r#"<a href="/">New Paste</a><meta charset="UTF-8"><table><tr><th>Name</th><th>Date</th><th>Link</th></tr>"#,
        );

        // Add a table row for each paste
        for (i, e) in ctx.app().read().unwrap().iter().enumerate() {
            out.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td><a href=\"/p/{}\">🔗</a></td></tr>",
                e.name,
                fmt_relative_time(e.time.elapsed().as_secs()),
                i
            ));
        }

        ctx
            .text(format!("{}</table>", out))
            .content(Content::HTML)
            .send()?;
        Ok(())
    });

    server.run()?;
    Ok(())
}

const TIME_UNITS: &[(&str, u16)] = &[
    ("second", 60),
    ("minute", 60),
    ("hour", 24),
    ("day", 30),
    ("month", 12),
    ("year", 0),
];

/// Turn relative number of seconds into a more readable relative time.
/// If the time is 0, now will be returned.
/// Ex 1 minute ago or 3 years ago
pub fn fmt_relative_time(secs: u64) -> String {
    if secs <= 1 {
        return "just now".into();
    }

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

const HTML: &str = r#"
<a href="/pastes">View Pastes</a>
<br />
<form action="/new-form" method="post">
    <input type="text" name="name" id="name" placeholder="Title">
    
    <br />
    <textarea id="body" name="body" rows="5" cols="33"></textarea>
    <input type="submit" value="Submit" />
</form>"#;
