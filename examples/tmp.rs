use std::fs::{self, File};

// use afire::extension::Logger;
use afire::prelude::*;

const PATH: &str = r#"D:\Movies\Breaking.Bad.S01-S05.1080p.BluRay.10bit.HEVC.6CH-MkvCage.ws\Breaking.Bad.S01.1080p.BluRay.10bit.HEVC.6CH-MkvCage.ws\Breaking.Bad.S01E01.Pilot.1080p.BluRay.10bit.HEVC.6CH-MkvCage.ws.mkv"#;

fn main() {
    let mut server = Server::<()>::new("localhost", 8080);
    // Logger::new().attach(&mut server);

    server.route(Method::POST, "/upload", |req| {
        println!("Receved {} bytes", req.body.len());
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

    server.start_threaded(5).unwrap();
}
