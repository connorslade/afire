use crate::{
    middleware::{MiddleResult, Middleware},
    Content, Header, HeaderType, Method, Request, Response,
};

/// Adds support for the [HTTP TRACE](https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/TRACE) method.
///
/// It echos the request (Status line + Headers) back to the client as the response body.
/// The `Cookie` header is excluded by default because it could contain sensitive information.
/// Read more about it in [RFC-9110](https://www.rfc-editor.org/rfc/rfc9110#TRACE).
pub struct Trace {
    exclude_headers: Vec<HeaderType>,
}

impl Trace {
    /// Create a new instance of the middleware.
    /// Note: The `Cookie` header is excluded by default because it could contain sensitive information.
    /// If you want to include it, use the [`include`] method.
    pub fn new() -> Self {
        Self {
            exclude_headers: vec![HeaderType::Cookie],
        }
    }

    /// Adds a header to the list of headers to exclude from the response.
    pub fn exclude(mut self, header: HeaderType) -> Self {
        self.exclude_headers.push(header);
        self
    }

    /// Adds a list of headers to the list of headers to exclude from the response.
    pub fn exclude_all(mut self, headers: &[HeaderType]) -> Self {
        self.exclude_headers.extend_from_slice(headers);
        self
    }

    /// Removes a header from the list of headers to exclude from the response.
    /// Likely to be used with for allowing the `Cookie` header to be sent, as it is excluded by default.
    pub fn include(mut self, header: HeaderType) -> Self {
        self.exclude_headers.retain(|h| *h != header);
        self
    }
}

impl Middleware for Trace {
    fn pre(&self, req: &mut Request) -> MiddleResult {
        if req.method != Method::TRACE {
            return MiddleResult::Continue;
        }

        let headers = req
            .headers
            .iter()
            .filter(|h| !self.exclude_headers.contains(&h.name))
            .map(Header::to_string)
            .fold(String::new(), |acc, i| acc + &i + "\r\n");

        let out = format!(
            "{} {} {}\r\n{}\r\n\r\n",
            req.method,
            req.path,
            req.version,
            &headers[..headers.len() - 2]
        );

        MiddleResult::Send(
            Response::new()
                .text(out)
                .content(Content::Custom("message/http")),
        )
    }
}

impl Default for Trace {
    fn default() -> Self {
        Self::new()
    }
}
