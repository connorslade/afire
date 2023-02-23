use std::{
    convert::TryFrom,
    fs::{self, File},
};

use afire::{
    extension::Date,
    extension::Logger,
    internal::encoding::{base64, sha1},
    multipart::MultipartData,
    prelude::*,
    trace,
    trace::{set_log_level, Level},
};

// File to download
const PATH: &str = r#"..."#;

fn main() {
    let mut server = Server::<()>::new("localhost", 8080);
    set_log_level(Level::Debug);
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

    Test.attach(&mut server);
    Date.attach(&mut server);
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
