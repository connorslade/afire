//! HTTP headers.

use std::{
    fmt::{self, Display},
    ops::{Deref, DerefMut},
};

use crate::{
    error::{ParseError, Result},
    internal::misc::filter_crlf,
};

/// Http header.
/// Has a name and a value.
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct Header {
    /// Name of the Header
    pub name: HeaderType,
    /// Value of the Header
    pub value: String,
}

/// Parameters for a header.
/// For example, the `charset` parameter in `Content-Type: text/html; charset=utf-8`.
/// ## Example
/// ```rust
/// # use afire::{Method, Server, Response, HeaderType};
/// # fn test(server: &mut Server) {
/// server.route(Method::GET, "/", |ctx| {
///     let header = ctx.req.headers.get_header(HeaderType::ContentType).unwrap();
///     let params = header.params();
///     let charset = params.get("charset").unwrap();
///     
///     ctx.text(format!("Charset: {}", charset))
///         .send()?;
///     Ok(())
/// });
/// # }
/// ```
pub struct HeaderParams<'a> {
    /// The value of the header.
    pub value: &'a str,
    /// The parameters of the header.
    params: Vec<[&'a str; 2]>,
}

/// Collection of headers.
/// Used within [`Request`](crate::Request) and [`Response`](crate::Response).
#[derive(Debug, Hash, Clone, PartialEq, Eq, Default)]
pub struct Headers(pub(crate) Vec<Header>);

impl Header {
    /// Make a new header from a name and a value.
    /// The name must implement `Into<HeaderType>`, so it can be a string or a [`HeaderType`].
    /// The value can be anything that implements `AsRef<str>`, including a String, or &str.
    ///
    /// Note: Neither the name nor the value may contain CRLF characters.
    /// They will be filtered out automatically.
    /// ## Example
    /// ```rust
    /// # use afire::Header;
    /// let header1 = Header::new("Content-Type", "text/html");
    /// let header2 = Header::new("Access-Control-Allow-Origin", "*");
    /// ```
    pub fn new(name: impl Into<HeaderType>, value: impl AsRef<str>) -> Header {
        Header {
            name: name.into(),
            value: filter_crlf(value.as_ref()),
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
    pub fn from_string(header: &str) -> Result<Header> {
        let mut split_header = header.splitn(2, ':');

        let name = split_header
            .next()
            .ok_or(ParseError::InvalidHeader)?
            .trim()
            .into();
        let value = split_header
            .next()
            .ok_or(ParseError::InvalidHeader)?
            .trim()
            .into();

        Ok(Header { name, value })
    }

    /// Get the parameters of the header.
    pub fn params(&self) -> HeaderParams {
        HeaderParams::new(self.value.as_str())
    }

    /// Checks if the header is a [forbidden header](https://developer.mozilla.org/en-US/docs/Glossary/Forbidden_header_name).
    pub fn is_forbidden(&self) -> bool {
        let name = self.name.to_string().to_ascii_lowercase();
        FORBIDDEN_HEADERS.iter().any(|x| {
            let xb = x.as_bytes();
            if xb[xb.len() - 1] == b'-' {
                return name.starts_with(x);
            }
            name == *x
        })
    }
}

impl<'a> HeaderParams<'a> {
    fn new(value: &'a str) -> Self {
        let mut params = Vec::new();

        let mut parts = value.split(';');
        let value = parts.next().unwrap_or_default();

        for i in parts {
            let mut split = i.splitn(2, '=');

            let Some(key) = split.next() else {
                break;
            };
            let Some(value) = split.next() else {
                break;
            };

            params.push([key.trim(), value.trim()]);
        }

        Self { value, params }
    }

    /// Checks if the header has the specified parameter.
    pub fn has(&self, name: impl AsRef<str>) -> bool {
        let name = name.as_ref();
        self.params.iter().any(|[key, _]| key == &name)
    }

    /// Gets the value of the specified parameter, returning `None` if it is not present.
    /// A parameter is a key-value pair that is separated by a semicolon and a space.
    pub fn get(&self, name: impl AsRef<str>) -> Option<&str> {
        let name = name.as_ref();
        self.params
            .iter()
            .find(|[key, _]| key == &name)
            .map(|[_, value]| *value)
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

impl<'a> Deref for HeaderParams<'a> {
    type Target = Vec<[&'a str; 2]>;

    fn deref(&self) -> &Self::Target {
        self.params.as_ref()
    }
}

impl<'a> DerefMut for HeaderParams<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.params.as_mut()
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

    /// Adds a header to the collection, using the specified name and value.
    /// See [`Headers::add_header`] for a version that takes a [`Header`] directly.
    /// ## Example
    /// ```rust
    /// # use afire::header::{Headers, HeaderType, Header};
    /// # fn test(headers: &mut Headers) {
    /// headers.add(HeaderType::ContentType, "text/html");
    /// # }
    pub fn add(&mut self, name: impl Into<HeaderType>, value: impl AsRef<str>) {
        self.0.push(Header::new(name, value));
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

    /// Adds a header to the collection.
    /// See [`Headers::add`] for a version that takes a name and value.
    /// ## Example
    /// ```rust
    /// # use afire::header::{Headers, HeaderType, Header};
    /// # fn test(headers: &mut Headers) {
    /// headers.add(HeaderType::ContentType, "text/html");
    /// # }
    pub fn add_header(&mut self, header: Header) {
        self.0.push(header);
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
/// Each header is in the format `name: value` amd separated by a carriage return and newline (`\r\n`).
pub(crate) fn headers_to_string(headers: &[Header]) -> String {
    let out = headers
        .iter()
        .map(Header::to_string)
        .fold(String::new(), |acc, i| acc + &i + "\r\n");

    out[..out.len() - 2].to_owned()
}

/// [Forbidden headers](https://developer.mozilla.org/en-US/docs/Glossary/Forbidden_header_name) are headers that should not be set by the user as they have special meaning in the HTTP protocol.
///
/// These headers are allowed to be set by the user in routes, but will throw an error if added to default headers.
/// This is because it may make sense to set these headers in a route in some cases, but it is never a good idea to set them in default headers.
///
/// Also note that entries ending with a dash (`-`) are prefixes, so any header that starts with that prefix is forbidden.
///
/// ## The Headers
/// - Accept-Charset
/// - Accept-Encoding
/// - Access-Control-Request-Headers
/// - Access-Control-Request-Method
/// - Connection
/// - Content-Length
/// - Cookie
/// - Date
/// - DNT
/// - Expect
/// - Host
/// - Keep-Alive
/// - Origin
/// - Permissions-Policy
/// - Proxy-
/// - Sec-
/// - Referer
/// - TE
/// - Trailer
/// - Transfer-Encoding
/// - Upgrade
/// - Via
pub const FORBIDDEN_HEADERS: &[&str] = &[
    "accept-charset",
    "accept-encoding",
    "access-control-request-headers",
    "access-control-request-method",
    "connection",
    "content-length",
    "cookie",
    "date",
    "dnt",
    "expect",
    "host",
    "keep-alive",
    "origin",
    "permissions-policy",
    "proxy-",
    "sec-",
    "referer",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
    "via",
];

macro_rules! headers {
    {
        $(
            $(#[$attr:meta])*
            $name:ident => $header:literal, $header_lower:literal
        ),*
    } => {
        /// HTTP header names.
        ///
        /// Headers are used for passing additional information in a HTTP message.
        #[derive(Debug, Hash, Clone, PartialEq, Eq)]
        pub enum HeaderType {
            $(
                $(#[$attr])*
                ///
                #[doc = concat!("[MDN Docs](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/", $header, ")")]
                $name
            ),*,
            /// Custom header type.
            /// Only used when the header type is unknown to afire.
            Custom(String),
        }

        impl HeaderType {
            fn from_str(s: &str) -> Self {
                use HeaderType::*;
                match s.to_ascii_lowercase().as_str() {
                    $($header_lower => $name),*,
                    _ => HeaderType::Custom(filter_crlf(s)),
                }
            }
        }

        impl Display for HeaderType {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                use HeaderType::*;
                f.write_str(match self {
                    $($name => $header),*,
                    HeaderType::Custom(c) => c,
                })
            }
        }
    }
}

headers! {
    /// Indicates what content types (MIME types) are acceptable for the client.
    Accept           => "Accept",            "accept",
    /// Indicates what character sets are acceptable for the client.
    AcceptCharset    => "Accept-Charset",    "accept-charset",
    /// Indicates what content encodings (usually compression algorithms) are acceptable for the client.
    AcceptEncoding   => "Accept-Encoding",   "accept-encoding",
    /// Indicates what languages are acceptable for the client.
    AcceptLanguage   => "Accept-Language",   "accept-language",
    /// Indicates whether the response can be shared with requesting code from the given origin.
    AccessControlAllowOrigin => "Access-Control-Allow-Origin", "access-control-allow-origin",
    /// Used to provide credentials that authenticate a user agent with a server, allowing access to a protected resource.
    Authorization    => "Authorization",     "authorization",
    /// Controls caching in browsers and shared caches like proxies and CDNs.
    CacheControl     => "Cache-Control",     "cache-control",
    /// Allows re-using a socket for multiple requests with `keep-alive`, or closing the sockets with `close`.
    Connection       => "Connection",        "connection",
    /// Lists the encodings that have been applied to the entity body.
    /// See [`HeaderType::AcceptEncoding`]
    ContentEncoding  => "Content-Encoding",  "content-encoding",
    /// An integer indicating the size of the entity body in bytes.
    /// This is only required when the body is not chunked.
    ContentLength    => "Content-Length",    "content-length",
    /// Indicates the media type of the entity body.
    /// This can be set on a response with the [`crate::Response::content`] method.
    ContentType      => "Content-Type",      "content-type",
    /// Contains cookies from the client.
    Cookie           => "Cookie",            "cookie",
    /// The date and time at which the message was originated.
    Date             => "Date",              "date",
    /// Sent with requests to indicate the host and port of the server to which the request is being sent.
    /// This allows for reverse proxies to forward requests to the correct server.
    Host             => "Host",              "host",
    /// Used with redirection status codes (301, 302, 303, 307, 308) to indicate the URL to redirect to.
    Location         => "Location",          "location",
    /// Contains the address of the webpage that linked to the resource being requested.
    /// Note the misspelling of referrer as 'referer' in the HTTP spec. so silly.
    Referer          => "Referer",           "referer",
    /// An identifier for a specific name / version of the web server software.
    #[doc = concat!("This is set to `afire/", env!("CARGO_PKG_VERSION"), "` by default.")]
    Server           => "Server",            "server",
    /// Used to send cookies from the server to the client.
    /// Its recommended to use the [`crate::SetCookie`] builder instead of this directly.
    SetCookie        => "Set-Cookie",        "set-cookie",
    /// Specifies the transfer encoding of the message body.
    TransferEncoding => "Transfer-Encoding", "transfer-encoding",
    /// Used to switch from HTTP to a different protocol on the same socket, often used for websockets.
    /// Note that afire *currently* does not have built-in support for websockets.
    Upgrade          => "Upgrade",           "upgrade",
    /// Contains information about the client application, operating system, vendor, etc. that is making the request.
    UserAgent        => "User-Agent",        "user-agent",
    /// A header added by proxies to track message forewords, avoid request loops, and identifying protocol capabilities.
    Via              => "Via",               "via",
    /// Defines the HTTP authentication methods ("challenges") that might be used to gain access to a specific resource.
    WWWAuthenticate  => "WWW-Authenticate",  "www-authenticate",
    /// A header often added by reverse proxies to allow web servers to know from which IP a request is originating.
    /// This is not an official HTTP header, but is still widely used.
    XForwardedFor    => "X-Forwarded-For",   "x-forwarded-for"
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
