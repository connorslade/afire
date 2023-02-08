//! HTTP headers.

use std::{
    fmt::{self, Display},
    ops::{Deref, DerefMut},
};

use crate::error::{ParseError, Result};

/// Http header.
/// Has a name and a value.
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct Header {
    /// Name of the Header
    pub name: HeaderType,

    /// Value of the Header
    pub value: String,
}

/// Collection of headers.
/// Used within [`Request`](crate::Request) and [`Response`](crate::Response).
#[derive(Debug, Hash, Clone, PartialEq, Eq, Default)]
pub struct Headers(pub(crate) Vec<Header>);

impl Header {
    /// Make a new header from a name and a value, which bolth mut implement AsRef<str>.
    /// ## Example
    /// ```rust
    /// # use afire::Header;
    /// let header1 = Header::new("Content-Type", "text/html");
    /// let header2 = Header::new("Access-Control-Allow-Origin", "*");
    /// ```
    pub fn new(name: impl Into<HeaderType>, value: impl AsRef<str>) -> Header {
        Header {
            name: name.into(),
            value: value.as_ref().to_owned(),
        }
    }

    /// Convert a string to a header.
    /// String must be in the format `name: value`, or an error will be returned.
    /// ## Example
    /// ```rust
    /// # use afire::{Header, HeaderType};
    /// let header1 = Header::new(HeaderType::ContentType, "text/html");
    /// let header2 = Header::from_string("Content-Type: text/html").unwrap();
    ///
    /// assert_eq!(header1, header2);
    /// ```
    pub fn from_string(header: impl AsRef<str>) -> Result<Header> {
        let header = header.as_ref();
        let mut split_header = header.splitn(2, ':');
        if split_header.clone().count() != 2 {
            return Err(ParseError::InvalidHeader.into());
        }

        let name = match split_header.next() {
            Some(i) => i.trim().to_string(),
            None => return Err(ParseError::InvalidHeader.into()),
        };

        let value = match split_header.next() {
            Some(i) => i.trim().to_string(),
            None => return Err(ParseError::InvalidHeader.into()),
        };

        Ok(Header {
            name: name.into(),
            value,
        })
    }
}

impl Deref for Headers {
    type Target = Vec<Header>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Headers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Headers {
    /// Checks if the request / response contains the specified header.
    /// ## Example
    /// ```rust
    /// # use afire::header::{Headers, HeaderType, Header};
    /// # fn test(headers: Headers) {
    /// if headers.has(HeaderType::UserAgent) {
    ///    println!("User-Agent header is present");
    /// }
    /// # }
    /// ```
    pub fn has(&self, name: impl Into<HeaderType>) -> bool {
        let name = name.into();
        self.iter().any(|x| x.name == name)
    }

    /// Gets the value of the specified header.
    /// If the header is not present, `None` is returned.
    /// ## Example
    /// ```rust
    /// # use afire::header::{Headers, HeaderType, Header};
    /// # fn test(headers: Headers) {
    /// if let Some(user_agent) = headers.get(HeaderType::UserAgent) {
    ///   println!("User-Agent: {}", user_agent);
    /// }
    /// # }
    /// ```
    pub fn get(&self, name: impl Into<HeaderType>) -> Option<&str> {
        let name = name.into();
        self.iter()
            .find(|x| x.name == name)
            .map(|x| x.value.as_str())
    }

    /// Gets the value of the specified header as a mutable reference.
    /// If the header is not present, `None` is returned.
    /// See [`Headers::get`] for a non-mutable version.
    pub fn get_mut(&mut self, name: impl Into<HeaderType>) -> Option<&mut String> {
        let name = name.into();
        self.iter_mut()
            .find(|x| x.name == name)
            .map(|x| &mut x.value)
    }

    /// Gets the specified header.
    /// If the header is not present, `None` is returned.
    pub fn get_header(&self, name: impl Into<HeaderType>) -> Option<&Header> {
        let name = name.into();
        self.iter().find(|x| x.name == name)
    }

    /// Gets the specified header as a mutable reference.
    /// If the header is not present, `None` is returned.
    /// See [`Headers::get_header`] for a non-mutable version.
    pub fn get_header_mut(&mut self, name: impl Into<HeaderType>) -> Option<&mut Header> {
        let name = name.into();
        self.iter_mut().find(|x| x.name == name)
    }
}

impl fmt::Display for Header {
    /// Convert a header to a string
    /// In format: `name: value`.
    /// ## Example
    /// ```rust
    /// # use afire::{Header, HeaderType};
    /// let header1 = Header::new(HeaderType::ContentType, "text/html");
    /// assert_eq!(header1.to_string(), "Content-Type: text/html");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

/// Stringify a Vec of headers.
/// Each header is in the format `name: value` amd separated by a carrage return and newline (`\r\n`).
pub(crate) fn headers_to_string(headers: &[Header]) -> String {
    let out = headers
        .iter()
        .map(Header::to_string)
        .fold(String::new(), |acc, i| acc + &i + "\r\n");

    out[..out.len() - 2].to_owned()
}

// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers
/// Common HTTP headers.
/// Just the 'common' ones, which are ones that I use semi-frequently, or that are used internally.
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum HeaderType {
    /// Indecates what content types (MIME types) are acceptable for the client.
    /// ([MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept))
    Accept,
    /// Indecates what character sets are acceptable for the client.
    /// ([MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Charset))
    AcceptCharset,
    /// Indecates what content encodings (usally compression algorithms) are acceptable for the client.
    /// ([MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Encoding))
    AcceptEncoding,
    /// Indecates what languages are acceptable for the client.
    /// ([MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Language))
    AcceptLanguage,
    /// Allows re-useing a socket for multiple requests with `keep-alive`, or closing the scoket with `close`.
    /// ([MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection))
    Connection,
    /// Lists the encodings that have been applied to the entity body.
    /// See [`HeaderType::AcceptEncoding`]
    /// ([MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Encoding))
    ContentEncoding,
    /// An integer indicating the size of the entity body in bytes.
    /// This is only required when the body is not chunked.
    /// ([MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Length))
    ContentLength,
    /// Indecates the media type of the entity body.
    /// This can be set on a response with the [`crate::Response::content`] method.
    /// ([MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Type))
    ContentType,
    /// Contains cookies from the client.
    /// ([MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cookie))
    Cookie,
    /// The date and time at which the message was originated.
    /// ([MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Date))
    Date,
    /// Sent with requests to indicate the host and port of the server to which the request is being sent.
    /// This allows for reverse proxies to forward requests to the correct server.
    /// ([MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Host))
    Host,
    /// Used with redirection status codes (301, 302, 303, 307, 308) to indicate the URL to redirect to.
    /// ([MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Location))
    Location,
    /// Contains the address of the webpage that linked to the resource being requested.
    /// Note the misspelling of referrer as 'referer' in the HTTP spec.
    /// ([MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Referer))
    Referer,
    /// An identifier for a specific name / version of the web server software.
    /// This is set to `afire/VERSION` by default.
    /// ([MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Server))
    Server,
    /// Used to send cookies from the server to the client.
    /// Its recommended to use the [`crate::SetCookie`] builder instead of this directly.
    /// ([MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie))
    SetCookie,
    /// Specifies the transfer encoding of the message body.
    /// ([MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Transfer-Encoding))
    TransferEncoding,
    /// Used to switch from HTTP to a different protocol on the same socket, often used for websockets.
    /// Note that afire *currently* does not have built-in support for websockets.
    /// ([MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Upgrade))
    Upgrade,
    /// Contains infomation about the client application, operating system, vendor, etc. that is making the request.
    /// ([MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/User-Agent))
    UserAgent,
    /// A header added by proxies to track message forewards, avoid request loops, and identifying protocol capabilities.
    /// ([MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Via))
    Via,
    /// Any other header that is not in this enum.
    Custom(String),
}

impl From<&HeaderType> for HeaderType {
    fn from(s: &HeaderType) -> Self {
        s.to_owned()
    }
}

impl<T: AsRef<str>> From<T> for HeaderType {
    fn from(s: T) -> Self {
        HeaderType::from_str(s.as_ref())
    }
}

impl HeaderType {
    fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "accept" => HeaderType::Accept,
            "accept-charset" => HeaderType::AcceptCharset,
            "accept-encoding" => HeaderType::AcceptEncoding,
            "accept-language" => HeaderType::AcceptLanguage,
            "connection" => HeaderType::Connection,
            "content-encoding" => HeaderType::ContentEncoding,
            "content-length" => HeaderType::ContentLength,
            "content-type" => HeaderType::ContentType,
            "cookie" => HeaderType::Cookie,
            "date" => HeaderType::Date,
            "host" => HeaderType::Host,
            "location" => HeaderType::Location,
            "referer" => HeaderType::Referer,
            "server" => HeaderType::Server,
            "set-cookie" => HeaderType::SetCookie,
            "transfer-encoding" => HeaderType::TransferEncoding,
            "upgrade" => HeaderType::Upgrade,
            "user-agent" => HeaderType::UserAgent,
            "via" => HeaderType::Via,
            _ => HeaderType::Custom(s.to_string()),
        }
    }
}

impl Display for HeaderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                HeaderType::Accept => "Accept",
                HeaderType::AcceptCharset => "Accept-Charset",
                HeaderType::AcceptEncoding => "Accept-Encoding",
                HeaderType::AcceptLanguage => "Accept-Language",
                HeaderType::Connection => "Connection",
                HeaderType::ContentEncoding => "Content-Encoding",
                HeaderType::ContentLength => "Content-Length",
                HeaderType::ContentType => "Content-Type",
                HeaderType::Cookie => "Cookie",
                HeaderType::Date => "Date",
                HeaderType::Host => "Host",
                HeaderType::Location => "Location",
                HeaderType::Referer => "Referer",
                HeaderType::Server => "Server",
                HeaderType::SetCookie => "Set-Cookie",
                HeaderType::TransferEncoding => "Transfer-Encoding",
                HeaderType::Upgrade => "Upgrade",
                HeaderType::UserAgent => "User-Agent",
                HeaderType::Via => "Via",
                HeaderType::Custom(s) => s,
            }
        )
    }
}
