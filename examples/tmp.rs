use std::{time::Instant, io::Read};

use afire::{Server, Response};

fn main() {
    let mut server = Server::<()>::new([127, 0, 0, 1], 8080);

    server.route(afire::Method::GET, "/test", |_| {
        Response::new().stream(TestStream::new())
    });

    server.start().unwrap();
}

struct TestStream {
    count: usize,
    last: Instant,
}

impl TestStream {
    fn new() -> Self {
        Self {
            count: 0,
            last: Instant::now(),
        }
    }
}

impl Read for TestStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.count >= 10 {
            return Ok(0);
        }

        let elapsed = self.last.elapsed().as_secs_f64();
        if elapsed < 1.0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "Not enough time has elapsed.",
            ));
        }

        self.last = Instant::now();
        self.count += 1;

        let data = format!("{{\"count\": {}}}", self.count);
        let bytes = data.as_bytes();
        let len = bytes.len().min(buf.len());
        buf[..len].copy_from_slice(&bytes[..len]);
        Ok(len)
    }
}