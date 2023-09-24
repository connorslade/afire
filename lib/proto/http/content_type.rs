//! Common MIME types for HTTP responses.

use crate::{headers::ContentType, Header};

use super::mime::{self, Mime};

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
    #[rustfmt::skip]
    pub fn as_type(&self) -> Mime {
        match self {
            Content::HTML => mime::HTML,
            Content::TXT  => mime::TEXT,
            Content::CSV  => mime::CSV,
            Content::JSON => mime::JSON,
            Content::XML  => mime::XML,
            Content::Custom(i) => (*i).into(),
        }
    }
}

impl From<Content<'_>> for Header {
    // Convert Content to a Content-Type Header
    fn from(x: Content<'_>) -> Self {
        ContentType::new(x.as_type()).into()
    }
}
