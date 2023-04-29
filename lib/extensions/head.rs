use crate::{
    middleware::{MiddleResult, Middleware},
    response::ResponseBody,
    HeaderType, Method, Request, Response,
};

/// Middleware to add support for the HTTP [HEAD] method.
/// It does this by changing the method to [GET] and adding a special header (`afire::head`).
/// Once the response is processed by the normal route handler, the middleware will check if the header is present.
/// If it is, any body data will be discarded and the [Content-Length] header will be added, if it is not already present.
/// On static responses, the length is already known, but with streaming responses, the stream will be read to the end to get the length (by default).
pub struct Head {
    /// Whether to add the Content-Length header to streaming responses.
    /// This is important because to get the length of a stream, it must be read to the end, which could be slow or impossible in some cases.
    /// By default this option is enabled.
    streaming: bool,
}

impl Head {
    /// Create a new instance of the middleware.
    pub fn new() -> Self {
        Self { streaming: true }
    }

    /// Set whether to add the Content-Length header to streaming responses.
    /// ## Example
    /// ```
    /// # use afire::extension::Head;
    /// let head = Head::new().streaming(false);
    /// ```
    pub fn streaming(mut self, streaming: bool) -> Self {
        self.streaming = streaming;
        self
    }
}

impl Middleware for Head {
    fn pre(&self, req: &mut Request) -> MiddleResult {
        if req.method != Method::HEAD {
            return MiddleResult::Continue;
        }

        req.method = Method::GET;
        req.headers.add("afire::head", "true");
        MiddleResult::Continue
    }

    fn post(&self, req: &Request, res: &mut Response) -> MiddleResult {
        if !req.headers.has("afire::head") {
            return MiddleResult::Continue;
        }

        let len = match &mut res.data {
            _ if res.headers.has(HeaderType::ContentLength) => None,
            ResponseBody::Static(d) => Some(d.len()),
            ResponseBody::Stream(s) if self.streaming => {
                let mut buf = Vec::new();
                s.get_mut().read_to_end(&mut buf).unwrap();
                Some(buf.len())
            }
            _ => None,
        };

        if let Some(i) = len {
            res.headers.add(HeaderType::ContentLength, i.to_string());
        }
        res.data = ResponseBody::empty();
        MiddleResult::Continue
    }
}
