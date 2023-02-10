//! A delightfully 90s website to store and view quotes
//! This example is slightly more advanced than the pastebin one because it uses a file to save the quotes.
//! In a real project you can probably use dependencies like `rusqlite` for a proper database.
//! But hey, more examples cant hurt!

use std::{
    collections::HashMap,
    fs,
    net::Ipv4Addr,
    path::PathBuf,
    sync::RwLock,
    time::{SystemTime, UNIX_EPOCH},
};

use afire::{
    extension::date::imp_date,
    internal::encoding::{decode_url, encode_url},
    trace,
    trace::{set_log_level, Level},
    Content, HeaderType, Method, Query, Response, Server, Status,
};

struct App {
    path: PathBuf,
    quotes: RwLock<HashMap<String, Quote>>,
}

struct Quote {
    name: String,
    value: String,
    date: u64,
}

fn main() {
    set_log_level(Level::Trace);
    let app = App::new(PathBuf::from("quotes.txt"));
    app.load();

    let mut server = Server::new(Ipv4Addr::LOCALHOST, 8080).state(app);

    // Route to serve the homepage (page that has add quote form)
    server.route(Method::GET, "/", |_| {
        Response::new()
            .text(String::new() + HEADER + HOME)
            .content(Content::HTML)
    });

    // Route to handle creating new quotes.
    // After successful creation the user will be redirected to the new quotes page.
    server.stateful_route(Method::POST, "/api/new", |app, req| {
        let form = Query::from_body(&String::from_utf8_lossy(&req.body));
        let name =
            decode_url(form.get("author").expect("No author supplied")).expect("Invalid author");
        let body =
            decode_url(form.get("quote").expect("No quote supplied")).expect("Invalid quote");

        let quote = Quote {
            name,
            value: body,
            date: now(),
        };
        let mut quotes = app.quotes.write().unwrap();
        let id = quotes.len();
        quotes.insert(id.to_string(), quote);
        drop(quotes);
        trace!(Level::Trace, "Added new quote #{id}");

        app.save();
        Response::new()
            .status(Status::SeeOther)
            .header(HeaderType::Location, format!("/quote/{id}"))
            .text("Redirecting to quote page.")
    });

    server.stateful_route(Method::GET, "/quote/{id}", |app, req| {
        let id = req.param("id").unwrap();
        if id == "undefined" {
            return Response::new();
        }

        let id = id.parse::<usize>().expect("ID is not a valid integer");
        let quotes = app.quotes.read().unwrap();
        if id >= quotes.len() {
            return Response::new()
                .status(Status::NotFound)
                .text(format!("No quote with the id {id} was found."));
        }

        let quote = quotes.get(&id.to_string()).unwrap();
        Response::new().content(Content::HTML).text(
            String::new()
                + HEADER
                + &QUOTE
                    .replace("{QUOTE}", &quote.value)
                    .replace("{AUTHOR}", &quote.name)
                    .replace("{TIME}", &imp_date(quote.date)),
        )
    });

    server.stateful_route(Method::GET, "/quotes", |app, _req| {
        let mut out = String::from(HEADER);
        out.push_str("<ul>");
        for i in app.quotes.read().unwrap().iter() {
            out.push_str(&format!(
                "<li><a href=\"/quote/{}\">\"{}\" - {}</a></li>\n",
                i.0, i.1.name, i.1.value
            ));
        }

        Response::new().text(out + "</ul>").content(Content::HTML)
    });

    // Note: In a production application you may want to multithread the server with the Server::start_threaded method.
    server.start().unwrap();
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

impl App {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            quotes: RwLock::new(HashMap::new()),
        }
    }

    fn load(&self) {
        if !self.path.exists() {
            trace!(Level::Trace, "No save file found. Skipping loading.");
            return;
        }

        let data = fs::read_to_string(&self.path).unwrap();
        let mut quotes = self.quotes.write().unwrap();
        quotes.clear();

        for i in data.lines() {
            let (name, quote) = i.split_once(':').unwrap();
            if let Some(i) = Quote::load(quote) {
                quotes.insert(name.to_owned(), i);
                continue;
            }
            trace!(Level::Error, "Error loading entry");
        }

        trace!("Loaded {} entries", quotes.len());
    }

    fn save(&self) {
        trace!(Level::Trace, "Saving quotes");
        let mut out = String::new();

        for i in self.quotes.read().unwrap().iter() {
            out.push_str(&format!("{}:{}\n", i.0, i.1.save()));
        }

        fs::write(&self.path, out).unwrap();
    }
}

impl Quote {
    fn save(&self) -> String {
        format!(
            "{}:{}:{}",
            encode_url(&self.name),
            encode_url(&self.value),
            self.date
        )
    }

    fn load(line: &str) -> Option<Self> {
        let mut parts = line.split(':');
        let name = decode_url(parts.next()?).unwrap();
        let value = decode_url(parts.next()?).unwrap();
        let date = parts.next()?.parse().ok()?;

        Some(Self { name, value, date })
    }
}

// Define webpage sources
// In all of my real applications, the web data is put in a web/ directory and served with the ServeStatic middleware.
// Im just embedding it in the code here to keep the example all contained in one file, please don't really do this.
// If you want to see some examples of some real afire applications checkout the 'afire hub' at https://connorcode.com/writing/afire.

const HEADER: &str = r#"
<a href="/">New Quote</a> •
<a href="/quotes">All Quotes</a>
"#;

// Note: When submitting the form it will send a POST to /api/new
const HOME: &str = r#"
<form method="post" action="/api/new">
    <label for="author">Author:</label>
    <input type="text" name="author" required>
    <br>
    <label for="quote">Quote:</label>
    <textarea name="quote" id="quote" cols="30" rows="4"></textarea>
    <br>
    <input type="submit" value="Submit">
</form>
"#;

const QUOTE: &str = r#"
<p>"{QUOTE}"</p>
<p> - {AUTHOR} ({TIME})</p>
"#;
