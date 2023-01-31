/// Common MIME types
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
