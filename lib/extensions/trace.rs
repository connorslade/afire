use crate::{
    header::headers_to_string,
    middleware::{MiddleResult, Middleware},
    Method, Request, Response,
};

pub struct Trace;

impl Middleware for Trace {
    fn pre(&self, req: &mut Request) -> MiddleResult {
        if req.method != Method::TRACE {
            return MiddleResult::Continue;
        }

        let mut out = format!(
            "{} {} {}\r\n{}\r\n\r\n",
            req.method,
            req.path,
            req.version,
            headers_to_string(&req.headers)
        )
        .as_bytes()
        .to_vec();
        out.extend(req.body.iter());

        MiddleResult::Send(Response::new().bytes(&out))
    }
}
