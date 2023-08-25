// Note: This module is intended for internal testing only

use std::{
    convert::TryFrom,
    error::Error,
    fmt::Arguments,
    fs::{self, File},
    io::{self, Read},
    sync::{mpsc::sync_channel, Arc, RwLock},
    thread,
    time::Duration,
};

use afire::{
    extensions::{Date, Head, Logger, RedirectResponse, RouteShorthands, ServeStatic, Trace},
    internal::sync::{ForceLockMutex, ForceLockRwLock},
    multipart::MultipartData,
    prelude::*,
    route::RouteContext,
    trace,
    trace::DefaultFormatter,
    trace::{set_log_formatter, set_log_level, Formatter, Level},
    websocket::TxType,
};

// File to download
const PATH: &str = r#"..."#;
const FILE_TYPE: &str = "...";

fn main() -> Result<(), Box<dyn Error>> {
    let mut server = Server::<()>::new("localhost", 8081).workers(4);
    set_log_level(Level::Debug);
    set_log_formatter(LogFormatter);
    Logger::new().attach(&mut server);

    server.get("/**", |ctx| {
        ctx.text("Not found! UWU?")
            .status(Status::NotFound)
            .send()?;
        Ok(())
    });

    server.get("/yt/{id}", |ctx| {
        let id = ctx.param("id");
        ctx.redirect(format!("https://youtube.com/watch?v={id}"))
            .send()?;
        Ok(())
    });

    server.route(Method::POST, "/upload", |ctx| {
        let content_type = ctx
            .req
            .headers
            .get(HeaderType::ContentType)
            .context("No content type")?;
        println!("Received {} bytes", ctx.req.body.len());
        ctx.bytes(&**ctx.req.body)
            .content(Content::Custom(content_type))
            .send()?;
        Ok(())
    });

    server.route(Method::GET, "/download", |ctx| {
        let data = fs::read(PATH).with_context(|| format!("File {PATH} not found!"))?;
        ctx.bytes(data).content(Content::Custom(FILE_TYPE)).send()?;
        Ok(())
    });

    server.route(Method::GET, "/download-stream", |ctx| {
        let stream = File::open(PATH).with_context(|| format!("File {PATH} not found!"))?;
        ctx.stream(stream)
            .content(Content::Custom(FILE_TYPE))
            .send()?;
        Ok(())
    });

    // let data = fs::read(PATH).with_context(|| format!("File {PATH} not found!"))?;
    // server.route(Method::GET, "/download-in-mem", move |ctx| {
    //     ctx.bytes(&*data)
    //         .content(Content::Custom(FILE_TYPE))
    //         .send()?;
    //     Ok(())
    // });

    server.route(Method::GET, "/info", |ctx| {
        let addr = ctx.req.socket.force_lock().peer_addr()?;
        let user_agent = ctx
            .req
            .headers
            .get(HeaderType::UserAgent)
            .context("No User-Agent supplied.")?;

        ctx.text(format!("{addr}: {user_agent}"))
            .content(Content::TXT)
            .send()?;

        Ok(())
    });

    server.route(Method::POST, "/file-upload", |ctx| {
        let multipart = MultipartData::try_from(&*ctx.req)?;
        let entry = multipart.get("file").context("No `file` section.")?;

        ctx.text(format!(
            "Received file `{}` ({}b)",
            entry.filename.as_ref().context("File has no name.")?,
            entry.data.len()
        ))
        .send()?;

        Ok(())
    });

    // No-copy file echo
    server.route(Method::POST, "/raw-upload", |ctx| {
        let body = ctx.req.body.clone();
        ctx.stream(Cursor::new(body))
            .content(Content::Custom(
                ctx.req
                    .headers
                    .get(HeaderType::ContentType)
                    .context("No Content-Type")?,
            ))
            .send()?;

        Ok(())
    });

    server.route(Method::GET, "/sse", |ctx| {
        let stream = ctx.sse()?;
        stream.set_retry(10_000);

        let mut start = 0;
        if let Some(i) = stream.last_index {
            stream.send_id("update", i, format!("Got last ID of `{i}`"));
            start = i + 1;
        }

        for i in 0..10 {
            stream.send_id("update", start + i, format!("eggs, are cool {}", start + i));
            thread::sleep(Duration::from_secs(1));
        }

        stream.close();
        println!("Closed stream");

        Ok(())
    });

    server.route(Method::GET, "/ws", |ctx| {
        let (tx, rx) = ctx.ws()?.split();

        tx.send("Hello, world!");

        thread::scope(|s| {
            s.spawn(|| {
                for i in 0.. {
                    if !tx.is_open() {
                        break;
                    }

                    thread::sleep(Duration::from_secs(3));
                    println!("Sending from another thread #{i}");
                    tx.send(format!("Hello from another thread #{i}"));
                }
                println!("Closing - Tx");
            });

            for i in rx.into_iter() {
                match i {
                    TxType::Close => break,
                    TxType::Text(t) => {
                        println!("Received: {}", t);
                        tx.send(format!("Received: {}", t));
                    }
                    TxType::Binary(b) => {
                        println!("Received: {:?}", b);
                        tx.send(format!("Received: {:?}", b));
                    }
                }
            }
            println!("Closing - Rx");
        });

        Ok(())
    });

    let users = Arc::new(RwLock::new(Vec::new()));
    server.route(Method::GET, "/chat", move |ctx| {
        let (ws_tx, ws_rx) = ctx.ws()?.split();
        let (tx, rx) = sync_channel(10);
        users.force_write().push(tx);

        thread::scope(|s| {
            s.spawn(move || {
                for i in rx {
                    if !ws_tx.is_open() {
                        break;
                    }

                    ws_tx.send(i);
                }
            });

            for i in ws_rx.into_iter() {
                match i {
                    TxType::Close => break,
                    TxType::Binary(_) => {}
                    TxType::Text(t) => {
                        users.force_read().iter().for_each(|u| {
                            u.send(t.to_owned()).unwrap();
                        });
                    }
                }
            }
        });

        Ok(())
    });

    server.route(Method::GET, "/", |ctx| {
        // let _ = File::open("index.html")
        //     .context("Failed to open file")
        //     .status(Status::InternalServerError)?;

        let threads = ctx.server.thread_pool.threads();
        let thread = ctx.server.thread_pool.current_thread().unwrap();
        ctx.text(format!("Ok!\nThreads: {threads}\nCurrent Thread: {thread}"))
            .header(HeaderType::ContentType, "text/plain")
            .send()?;

        Ok(())
    });

    server.route(Method::GET, "/nil", |ctx| {
        let socket = ctx.req.socket.clone();
        ctx.guarantee_will_send();

        thread::spawn(move || {
            thread::sleep(Duration::from_secs(3));
            trace!("Sending from another thread");
            Response::new()
                .text("Hello from another thread")
                .write(socket, &[])
                .unwrap();
        });

        Ok(())
    });

    server.route(Method::GET, "/greet", |ctx| {
        let name = ctx.req.query.get("name").context("No name provided")?;
        ctx.text(format!("Hello, {}!", name))
            .content(Content::TXT)
            .send()?;

        Ok(())
    });

    server.route(Method::GET, "/shutdown", |ctx| {
        ctx.server.shutdown();
        Ok(())
    });

    server.route(Method::GET, "/panic", |_ctx| panic!("slayyyter"));

    server.route(Method::GET, "/echo-headers", |ctx| {
        let header = ctx
            .req
            .headers
            .get("Header")
            .context("No `Header` header")?;

        ctx.header("Header", header.replace(r"\n", "\n"))
            .text("Ok!")
            .send()?;
        Ok(())
    });

    server.route(Method::GET, "/slow", |ctx| {
        thread::sleep(Duration::from_secs(5));
        ctx.text("Waited 5 seconds.").send()?;
        Ok(())
    });

    server.route(Method::GET, "/socket", |ctx| {
        let id = ctx.req.socket.id;
        ctx.text(format!("Socket #{id}")).send()?;
        Ok(())
    });

    server.route(Method::GET, "/ne", |ctx| {
        ctx.guarantee_will_send();
        let socket = ctx.req.socket.clone();
        thread::spawn(move || {
            Response::new()
                .text("Hello from another thread")
                .write(socket, &[])
                .unwrap();
        });
        Ok(())
    });

    server.route(Method::GET, "/send-2", |ctx| {
        ctx.text("1").send()?;
        ctx.text("2").send()?;
        Ok(())
    });

    Test.attach(&mut server);
    Date.attach(&mut server);
    Trace::new().attach(&mut server);
    Head::new().attach(&mut server);
    ServeStatic::new("../misc/scripts/ayesha").attach(&mut server);
    Logger::new().attach(&mut server);
    server.run()?;

    Ok(())
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

    fn end(&self, req: Arc<Request>, _res: &Response) {
        if req.path.contains("hello") {
            println!("End: {}", req.path);
        }
    }
}

struct LogFormatter;

impl Formatter for LogFormatter {
    fn format(&self, level: Level, color: bool, msg: Arguments) {
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
