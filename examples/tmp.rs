// Note: This module is intended for internal testing only

use std::{
    collections::HashMap,
    convert::TryFrom,
    fs::{self, File},
    io::{self, Read},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use afire::{
    extension::{Date, Head, Logger, Trace},
    internal::encoding::{base64, sha1},
    multipart::MultipartData,
    prelude::*,
    server_sent_events::{Event, ServerSentEvents},
    trace,
    trace::DefaultFormatter,
    trace::{set_log_formatter, set_log_level, Formatter, Level},
};

// File to download
const PATH: &str = r#"..."#;

fn main() {
    let mut server = Server::<()>::new("localhost", 8080);
    set_log_level(Level::Debug);
    set_log_formatter(LogFormatter);
    Logger::new().attach(&mut server);

    server.route(Method::POST, "/upload", |req| {
        println!("Received {} bytes", req.body.len());
        Response::new().bytes(&req.body)
    });

    server.route(Method::GET, "/download", |_| {
        let data = fs::read(PATH).unwrap();
        Response::new().bytes(&data)
    });

    server.route(Method::GET, "/download-stream", |_| {
        let stream = File::open(PATH).unwrap();
        Response::new().stream(stream)
    });

    server.route(Method::GET, "/", |req| {
        let mango = req.socket.lock().unwrap();
        let user_agent = req.headers.get(HeaderType::UserAgent).unwrap();
        println!("{}", mango.peer_addr().unwrap());
        Response::new().text(user_agent).content(Content::TXT)
    });

    server.route(Method::ANY, "/panic", |_| panic!("panic!"));

    server.route(Method::POST, "/file-upload", |req| {
        let multipart = MultipartData::try_from(req).unwrap();
        let entry = multipart.get("file").unwrap();

        Response::new().bytes(entry.data).content(Content::Custom(
            entry.headers.get(HeaderType::ContentType).unwrap(),
        ))
    });

    // No-copy file echo
    server.route(Method::POST, "/raw-upload", |req| {
        let body = req.body.clone();
        Response::new()
            .stream(Cursor::new(body))
            .content(Content::Custom(
                req.headers.get(HeaderType::ContentType).unwrap(),
            ))
    });

    server.route(Method::GET, "/header-stat", |req| {
        let headers = req.headers.to_vec();
        let mut head_map = HashMap::new();

        for i in headers.iter() {
            head_map.insert(i.name.clone(), i);
        }

        let mut res = String::new();
        let start = Instant::now();
        for _ in 0..100000 {
            head_map.get(&HeaderType::UserAgent).unwrap();
        }
        let end = Instant::now();
        res.push_str(&format!(
            "HashMap: {}ns\n",
            end.duration_since(start).as_nanos()
        ));

        let start = Instant::now();
        for _ in 0..100000 {
            headers
                .iter()
                .find(|i| i.name == HeaderType::UserAgent)
                .unwrap();
        }
        let end = Instant::now();
        res.push_str(&format!(
            "Vec:     {}ns",
            end.duration_since(start).as_nanos()
        ));

        Response::new().text(res)
    });

    const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    server.route(Method::GET, "/ws", |req| {
        let ws_key = req.headers.get("Sec-WebSocket-Key").unwrap().to_owned();
        trace!("WS Key: {}", ws_key);
        let accept = base64::encode(&sha1::hash((ws_key + WS_GUID).as_bytes()));
        trace!("WS Accept: {}", accept);

        let mut upgrade = Response::new()
            .status(Status::SwitchingProtocols)
            .header(HeaderType::Upgrade, "websocket")
            .header(HeaderType::Connection, "Upgrade")
            .header("Sec-WebSocket-Accept", &accept)
            .header("Sec-WebSocket-Version", "13");
        upgrade.write(req.socket.clone(), &[]).unwrap();

        Response::end()
    });

    server.route(Method::GET, "/sse", |req| {
        let tx = req.sse().unwrap();

        for i in 0..10 {
            tx.send(Event::new("update").id(i).data("eggs, are cool"))
                .unwrap();
            thread::sleep(Duration::from_secs(1));
        }

        Response::end()
    });

    Test.attach(&mut server);
    Date.attach(&mut server);
    Trace::new().attach(&mut server);
    Head::new().attach(&mut server);
    server.start_threaded(5).unwrap();
}

struct Test;

impl Middleware for Test {
    fn pre(&self, req: &mut Request) -> MiddleResult {
        if req.path.contains("hello") {
            println!("Pre: {}", req.path);
            return MiddleResult::Send(Response::new().text("Intercepted"));
        }

        MiddleResult::Continue
    }

    fn post(&self, req: &Request, _res: &mut Response) -> MiddleResult {
        if req.path.contains("hello") {
            println!("Post: {}", req.path);
        }

        MiddleResult::Continue
    }

    fn end(&self, req: &Request, _res: &Response) {
        if req.path.contains("hello") {
            println!("End: {}", req.path);
        }
    }
}

struct LogFormatter;

impl Formatter for LogFormatter {
    fn format(&self, level: Level, color: bool, msg: String) {
        DefaultFormatter.format(level, color, msg);
    }
}

struct Cursor {
    inner: Arc<Vec<u8>>,
    index: u64,
}

impl Cursor {
    fn new(inner: Arc<Vec<u8>>) -> Self {
        Self { inner, index: 0 }
    }

    fn remaining_slice(&self) -> &[u8] {
        let len = self.index.min(self.inner.as_ref().len() as u64);
        &self.inner.as_ref()[(len as usize)..]
    }
}

impl Read for Cursor {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = Read::read(&mut self.remaining_slice(), buf)?;
        self.index += n as u64;
        Ok(n)
    }
}
