use crate::Header;

/// Common MIME types for HTTP responses.
/// Used with the [`crate::Context::content`] and [`crate::Response::content`] methods.
/// ## Example
/// ```
/// # use afire::prelude::*;
/// # fn test(server: &mut Server) {
/// server.route(Method::GET, "/api/hello", |ctx| {
///     ctx.content(Content::JSON)
///         .text(r#"{"hello": "world"}"#)
///         .send()?;
///     Ok(())
/// });
/// # }
#[derive(Debug, PartialEq, Eq)]
pub enum Content<'a> {
    /// HTML - `text/html`
    HTML,
    /// TXT - `text/plain`
    TXT,
    /// CSV - `text/csv`
    CSV,
    /// JSON - `application/json`
    JSON,
    /// XML - `application/xml`
    XML,
    /// Custom Content Type
    Custom(&'a str),
}

impl Content<'_> {
    /// Get Content as a MIME Type
    pub fn as_type(&self) -> &str {
        match self {
            Content::HTML => "text/html",
            Content::TXT => "text/plain",
            Content::CSV => "text/csv",
            Content::JSON => "application/json",
            Content::XML => "application/xml",
            Content::Custom(i) => i,
        }
    }
}

impl From<Content<'_>> for Header {
    // Convert Content to a Content-Type Header
    fn from(x: Content<'_>) -> Self {
        Header::new(
            "Content-Type",
            format!(
                "{}{}",
                x.as_type(),
                if !matches!(x, Content::Custom(_)) {
                    "; charset=utf-8"
                } else {
                    ""
                }
            ),
        )
    }
}
