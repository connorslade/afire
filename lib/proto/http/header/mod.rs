//! HTTP headers.
//!
//! When using headers in afire, many functions take [`Into<Header>`][`Header#impl-Into<Header>-for-(T,+K)`] as parameters.
//! This allows you to use many different types as headers, including tuples, [`Header`]s, and *header structs*.
//!
//! ## Example
//! ```rust
//! # use afire::prelude::*;
//! # use afire::headers;
//! # fn test(server: &mut Server) {
//! server.route(Method::GET, "/", |ctx| {
//!     ctx.header(("X-Test", "Test")); // Set 'X-Test' header to 'Test'
//!     ctx.header(headers::Server::new("teapot")); // Set 'Server' header to 'teapot'
//!
//!     ctx.text("Hello World!").send()?;
//!     Ok(())
//! });
//! # }
//! ```
//!
//! ## Header Structs
//!
//! Header structs are structs that implement [`Into<Header>`][`Header#impl-Into<Header>-for-(T,+K)`] and make it easier to set headers.
//! afire currently has the following header structs:
//!
//! |[`AccessControlAllowOrigin`]|[`ContentLength`]|
//! | -------------------------- | --------------- |
//! |[`CacheControl`]            |[`Date`]         |
//! |[`Connection`]              |[`Location`]     |
//! |[`ContentEncoding`]         |[`Server`]       |

use std::{
    borrow::Cow,
    fmt::{self},
    ops::{Deref, DerefMut},
};

use crate::{
    error::{ParseError, Result},
    internal::misc::filter_crlf,
};

mod header_name;

pub use header_name::HeaderName;

/// Http header.
/// Has a name and a value.
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct Header {
    /// Name of the Header
    pub name: HeaderName,
    /// Value of the Header
    pub value: Cow<'static, str>,
}

/// Parameters for a header.
/// For example, the `charset` parameter in `Content-Type: text/html; charset=utf-8`.
/// ## Example
/// ```rust
/// # use afire::{Method, Server, Response, HeaderName};
/// # fn test(server: &mut Server) {
/// server.route(Method::GET, "/", |ctx| {
///     let header = ctx.req.headers.get_header(HeaderName::ContentType).unwrap();
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
    /// The name must implement `Into<HeaderName>`, so it can be a string or a [`HeaderName`].
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
    pub fn new(name: impl Into<HeaderName>, value: impl AsRef<str>) -> Header {
        Header {
            name: name.into(),
            value: Cow::Owned(filter_crlf(value.as_ref())),
        }
    }

    /// Convert a string to a header.
    /// String must be in the format `name: value`, or an error will be returned.
    /// ## Example
    /// ```rust
    /// # use afire::{Header, HeaderName};
    /// let header1 = Header::new(HeaderName::ContentType, "text/html");
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
            .to_owned();

        Ok(Header {
            name,
            value: Cow::Owned(value),
        })
    }

    /// Get the parameters of the header.
    pub fn params(&self) -> HeaderParams {
        HeaderParams::new(&self.value)
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
    /// # use afire::header::{Headers, HeaderName, Header};
    /// # fn test(headers: Headers) {
    /// if headers.has(HeaderName::UserAgent) {
    ///    println!("User-Agent header is present");
    /// }
    /// # }
    /// ```
    pub fn has(&self, name: impl Into<HeaderName>) -> bool {
        let name = name.into();
        self.iter().any(|x| x.name == name)
    }

    /// Adds a header to the collection, using the specified name and value.
    /// See [`Headers::add_header`] for a version that takes a [`Header`] directly.
    /// ## Example
    /// ```rust
    /// # use afire::header::{Headers, HeaderName, Header};
    /// # fn test(headers: &mut Headers) {
    /// headers.add(HeaderName::ContentType, "text/html");
    /// # }
    pub fn add(&mut self, name: impl Into<HeaderName>, value: impl AsRef<str>) {
        self.0.push(Header::new(name, value));
    }

    /// Gets the value of the specified header.
    /// If the header is not present, `None` is returned.
    /// ## Example
    /// ```rust
    /// # use afire::header::{Headers, HeaderName, Header};
    /// # fn test(headers: Headers) {
    /// if let Some(user_agent) = headers.get(HeaderName::UserAgent) {
    ///   println!("User-Agent: {}", user_agent);
    /// }
    /// # }
    /// ```
    pub fn get(&self, name: impl Into<HeaderName>) -> Option<&Cow<'static, str>> {
        let name = name.into();
        self.iter().find(|x| x.name == name).map(|x| &x.value)
    }

    /// Gets the value of the specified header as a mutable reference.
    /// If the header is not present, `None` is returned.
    /// See [`Headers::get`] for a non-mutable version.
    pub fn get_mut(&mut self, name: impl Into<HeaderName>) -> Option<&mut Cow<'static, str>> {
        let name = name.into();
        self.iter_mut()
            .find(|x| x.name == name)
            .map(|x| &mut x.value)
    }

    /// Adds a header to the collection.
    /// See [`Headers::add`] for a version that takes a name and value.
    /// ## Example
    /// ```rust
    /// # use afire::header::{Headers, HeaderName, Header};
    /// # fn test(headers: &mut Headers) {
    /// headers.add(HeaderName::ContentType, "text/html");
    /// # }
    pub fn add_header(&mut self, header: Header) {
        self.0.push(header);
    }

    /// Gets the specified header.
    /// If the header is not present, `None` is returned.
    pub fn get_header(&self, name: impl Into<HeaderName>) -> Option<&Header> {
        let name = name.into();
        self.iter().find(|x| x.name == name)
    }

    /// Gets the specified header as a mutable reference.
    /// If the header is not present, `None` is returned.
    /// See [`Headers::get_header`] for a non-mutable version.
    pub fn get_header_mut(&mut self, name: impl Into<HeaderName>) -> Option<&mut Header> {
        let name = name.into();
        self.iter_mut().find(|x| x.name == name)
    }
}

impl fmt::Display for Header {
    /// Convert a header to a string
    /// In format: `name: value`.
    /// ## Example
    /// ```rust
    /// # use afire::{Header, HeaderName};
    /// let header1 = Header::new(HeaderName::ContentType, "text/html");
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

impl<T: Into<HeaderName>, K: AsRef<str>> From<(T, K)> for Header {
    fn from(value: (T, K)) -> Self {
        Header::new(value.0, value.1)
    }
}
