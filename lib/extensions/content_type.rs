/// Common MIME types
pub enum Content {
    /// HTML - `text/html`
    HTML,
    /// CSS - `text/css`
    CSS,
    /// JS - `application/javascript`
    JS,
    /// PNG - `image/png`
    PNG,
    /// JPEG - `image/jpeg`
    JPG,
    /// GIF - `image/gif`
    GIF,
    /// ICO - `image/x-icon`
    ICO,
    /// SVG - `image/svg+xml`
    SVG,
    /// TXT - `text/plain`
    TXT,
    /// Custom Content Type
    Custom(String),
}

impl Content {
    /// Get Content as a MIME Type
    pub fn as_type(&self) -> String {
        match self {
            Content::HTML => "text/html",
            Content::CSS => "text/css",
            Content::JS => "application/javascript",
            Content::PNG => "image/png",
            Content::JPG => "image/jpeg",
            Content::GIF => "image/gif",
            Content::ICO => "image/x-icon",
            Content::SVG => "image/svg+xml",
            Content::TXT => "text/plain",
            Content::Custom(i) => i,
        }
        .to_owned()
    }
}

// AAC,
// AVI,
// BIN,
// BMP,
// BZ,
// BZ2,
// CDA,
// CSV,
// EPUB,
// GZ,
// ICS,
// JAR,
// JSON,
// JSONLD,
// MIDI,
// MID,
// MJS,
// MP3,
// MP4,
// MPEG,
// OGA,
// OGV,
// OGX,
// OPUS,
// OTF,
// PDF,
// RAR,
// RTF,
// SH,
// SWF,
// TAR,
// TIF,
// TIFF,
// TS,
// TTF,
// WAV,
// WEBA,
// WEBM,
// WEBP,
// WOFF,
// WOFF2,
// XHTML,
// XML,
// ZIP
