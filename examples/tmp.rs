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
    internal::sync::ForceLockMutex,
    multipart::MultipartData,
    prelude::*,
    server_sent_events::ServerSentEventsExt,
    trace,
    trace::DefaultFormatter,
    trace::{set_log_formatter, set_log_level, Formatter, Level},
};

// File to download
const PATH: &str = r#"..."#;

fn main() {
    let mut server = Server::<()>::new("localhost", 8081);
    set_log_level(Level::Debug);
    set_log_formatter(LogFormatter);
    Logger::new().attach(&mut server);

    // server.route(Method::POST, "/upload", |req| {
    //     println!("Received {} bytes", req.body.len());
    //     Response::new().bytes(&req.body)
    // });

    // server.route(Method::GET, "/download", |_| {
    //     let data = fs::read(PATH).unwrap();
    //     Response::new().bytes(&data)
    // });

    // server.route(Method::GET, "/download-stream", |_| {
    //     let stream = File::open(PATH).unwrap();
    //     Response::new().stream(stream)
    // });

    // server.route(Method::GET, "/", |req| {
    //     let mango = req.socket.lock().unwrap();
    //     let user_agent = req.headers.get(HeaderType::UserAgent).unwrap();
    //     println!("{}", mango.peer_addr().unwrap());
    //     Response::new().text(user_agent).content(Content::TXT)
    // });

    // server.route(Method::ANY, "/panic", |_| panic!("panic!"));

    // server.route(Method::POST, "/file-upload", |req| {
    //     let multipart = MultipartData::try_from(req).unwrap();
    //     let entry = multipart.get("file").unwrap();

    //     Response::new().bytes(entry.data).content(Content::Custom(
    //         entry.headers.get(HeaderType::ContentType).unwrap(),
    //     ))
    // });

    // // No-copy file echo
    // server.route(Method::POST, "/raw-upload", |req| {
    //     let body = req.body.clone();
    //     Response::new()
    //         .stream(Cursor::new(body))
    //         .content(Content::Custom(
    //             req.headers.get(HeaderType::ContentType).unwrap(),
    //         ))
    // });

    // server.route(Method::GET, "/header-stat", |req| {
    //     let headers = req.headers.to_vec();
    //     let mut head_map = HashMap::new();

    //     for i in headers.iter() {
    //         head_map.insert(i.name.clone(), i);
    //     }

    //     let mut res = String::new();
    //     let start = Instant::now();
    //     for _ in 0..100000 {
    //         head_map.get(&HeaderType::UserAgent).unwrap();
    //     }
    //     let end = Instant::now();
    //     res.push_str(&format!(
    //         "HashMap: {}ns\n",
    //         end.duration_since(start).as_nanos()
    //     ));

    //     let start = Instant::now();
    //     for _ in 0..100000 {
    //         headers
    //             .iter()
    //             .find(|i| i.name == HeaderType::UserAgent)
    //             .unwrap();
    //     }
    //     let end = Instant::now();
    //     res.push_str(&format!(
    //         "Vec:     {}ns",
    //         end.duration_since(start).as_nanos()
    //     ));

    //     Response::new().text(res)
    // });

    // server.route(Method::GET, "/ws", |req| {
    //     let stream = req.ws().unwrap();

    //     loop {
    //         println!("Sending...");
    //         stream.send("hello world");
    //         thread::sleep(Duration::from_secs(5));
    //     }

    //     // Response::end()
    // });

    // server.route(Method::GET, "/sse", |req| {
    //     let stream = req.sse().unwrap();
    //     stream.set_retry(10_000);

    //     let mut start = 0;
    //     if let Some(i) = stream.last_index {
    //         stream.send_id("update", i, format!("Got last ID of `{i}`"));
    //         start = i + 1;
    //     }

    //     for i in 0..10 {
    //         stream.send_id("update", start + i, format!("eggs, are cool {}", start + i));
    //         thread::sleep(Duration::from_secs(1));
    //     }

    //     Response::end()
    // });

    server.route(Method::GET, "/", |ctx| {
        // let file = File::open("index.html")?;
        let threads = ctx.server.thread_pool.threads();
        ctx.text(format!("Ok!\nThreads: {threads}")).send()?;
        Ok(())
    });

    server.route(Method::GET, "/nil", |ctx| {
        ctx.guarantee_will_send();
        let socket = ctx.req.socket.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(3));
            trace!("Sending from another thread");
            Response::new()
                .text("Hello from another thread")
                .write(socket, &[])
                .unwrap();
        });

        // thread::sleep(Duration::from_secs(4));

        Ok(())
    });
    server.route(Method::GET, "/panic", |_ctx| panic!());

    server.thread_pool.resize(10);

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
