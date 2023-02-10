use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::RwLock,
};

use afire::{
    internal::encoding::{decode_url, encode_url},
    trace,
    trace::{set_log_level, Level},
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
    // let app = A
}

impl App {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            quotes: RwLock::new(HashMap::new()),
        }
    }

    fn load(&self) {
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
