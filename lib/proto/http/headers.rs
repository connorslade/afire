//! Contains all the header structs that can be used with HTTP requests and responses.

use std::{borrow::Cow, fmt::Display};

use crate::{
    proto::http::mime::{self, Mime},
    Header, HeaderName,
};

pub use access_control_allow_origin::*;
pub use cache_control::*;
pub use connection::*;
pub use content_encoding::*;
pub use content_length::*;
pub use content_type::*;
pub use date::*;
pub use location::*;
pub use misc::*;
pub use server::*;
pub use vary::*;

mod access_control_allow_origin {
    use crate::internal::misc::filter_crlf;

    use super::*;

    /// `Access-Control-Allow-Origin` header for responses. ([Fetch Standard §3.2.3](https://fetch.spec.whatwg.org/#http-access-control-allow-origin))
    ///
    /// Response header that indicates whether the response can be shared with requesting code from the given origin.
    ///
    /// [MDN Docs](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Origin)
    pub enum AccessControlAllowOrigin {
        /// The resource can be shared with any origin.
        Any,
        /// The resource can be shared with requesting code from the same origin.
        /// If the server supports clients from multiple origins, it must return the correct origin for the client that made the request.
        Origin(Cow<'static, str>),
    }

    impl AccessControlAllowOrigin {
        /// Creates a new `AccessControlAllowOrigin::Origin` header with the given origin.
        pub fn origin(origin: impl Into<Cow<'static, str>>) -> Self {
            AccessControlAllowOrigin::Origin(origin.into())
        }
    }

    impl From<AccessControlAllowOrigin> for Header {
        fn from(value: AccessControlAllowOrigin) -> Self {
            let value = match value {
                AccessControlAllowOrigin::Any => "*".to_string(),
                AccessControlAllowOrigin::Origin(origin) => filter_crlf(&origin),
            };

            Header {
                name: HeaderName::AccessControlAllowOrigin,
                value: Cow::Owned(value),
            }
        }
    }
}

mod cache_control {
    use super::*;

    /// `Cache-Control` header for requests and responses. ([RFC 7234 §5.2](https://tools.ietf.org/html/rfc7234#section-5.2))
    ///
    /// Holds directives that control caching in browsers and shared caches line proxies and CDNs.
    ///
    /// [MDN Docs](https://developer.mozilla.org/en-US/docs/web/http/headers/Cache-Control)
    pub struct CacheControl {
        directives: Box<[CacheDirective]>,
    }

    /// Response directives for `Cache-Control`.
    ///
    /// [MDN Docs](https://developer.mozilla.org/en-US/docs/web/http/headers/Cache-Control#cache_directives)
    pub enum CacheDirective {
        /// `max-age=N`
        ///
        /// Indicates that the response remains fresh until N seconds after the response is generated.
        /// While the response is fresh, it can be served from cache without any interaction with the server.
        ///
        /// If a intermediate cache receives a response with a max-age directive and holds onto it for some time, it can use the `Age` header to tell the client to deduct that time from the freshness lifetime.
        MaxAge(u32),
        /// `s-maxage=N`
        ///
        /// Like [`CacheDirective::MaxAge`], but only applies to shared caches.
        SMaxage(u32),
        /// `no-cache`
        ///
        /// Indicates that the response can be stored in caches, but the response must be validated with the origin server before each reuse, even when the cache is disconnected from the origin server.
        NoCache,
        /// `no-store`
        ///
        /// Indicates that any caches of any kind (private or shared) should not store this response.
        NoStore,
        /// `no-transform`
        ///
        /// Indicates that any intermediary (regardless of whether it implements a cache) shouldn't transform the response contents.
        NoTransform,
        /// `must-revalidate`
        ///
        /// Indicates that the response can be stored in caches and can be reused while fresh. If the response becomes stale, it must be validated with the origin server before reuse.
        /// Typically, must-revalidate is used with max-age.
        ///
        /// HTTP allows caches to reuse stale responses when they are disconnected from the origin server.
        /// `must-revalidate` is a way to prevent this from happening - either the stored response is revalidated with the origin server or a 504 (Gateway Timeout) response is generated.
        MustRevalidate,
        /// `proxy-revalidate`
        ///
        /// Like [`CacheDirective::MustRevalidate`], but only applies to shared caches.
        ProxyRevalidate,
        /// `must-understand`
        ///
        /// Indicates that a cache should store the response only if it understands the requirements for caching based on status code.
        /// Should be coupled with `no-store` for fallback behavior.
        MustUnderstand,
        /// `private`
        ///
        /// Indicates that the response can be stored only in a private cache like local caches in browsers.
        ///
        /// You should add the private directive for user-personalized content, especially for responses received after login and for sessions managed via cookies.
        /// If you forget to add private to a response with personalized content, then that response can be stored in a shared cache and end up being reused for multiple users, which can cause personal information to leak.
        Private,
        /// `public`
        ///
        /// Indicates that the response can be stored in a shared cache.
        ///
        /// Responses for requests with Authorization header fields must not be stored in a shared cache; however, the public directive will cause such responses to be stored in a shared cache.
        Public,
        /// `immutable`
        ///
        /// Indicates that the response will not be updated while it's fresh.
        /// Usually used with `max-age`.
        Immutable,
        /// `stale-while-revalidate=N`
        ///
        /// Indicates that the cache could reuse a stale response while it re-validates it to a cache.
        StaleWhileRevalidate(u32),
        /// `stale-if-error`
        ///
        /// Indicates that the cache can reuse a stale response when an upstream server generates an error, or when the error is generated locally.
        /// Here, an error is considered any response with a status code of 500, 502, 503, or 504.
        StaleIfError(u32),
    }

    impl CacheControl {
        /// Creates a new `CacheControl` header with the given directives.
        /// ## Example
        /// ```
        /// # use afire::headers::{CacheControl, CacheDirective};
        /// CacheControl::new([CacheDirective::MaxAge(3600), CacheDirective::NoTransform]);
        /// ```
        pub fn new(directives: impl Into<Box<[CacheDirective]>>) -> Self {
            CacheControl {
                directives: directives.into(),
            }
        }

        /// Creates a new `CacheControl` header with just the `max-age` directive set to the given value.
        pub fn max_age(age: u32) -> Self {
            CacheControl::new([CacheDirective::MaxAge(age)])
        }

        /// Creates a new `CacheControl` header with just the `no-cache` directive set.
        pub fn no_cache() -> Self {
            CacheControl::new([CacheDirective::NoCache])
        }
    }

    impl From<CacheControl> for Header {
        fn from(value: CacheControl) -> Self {
            Header {
                name: HeaderName::CacheControl,
                value: Cow::Owned(comma_separated(value.directives, |x| x.value())),
            }
        }
    }

    impl CacheDirective {
        /// Gets the value of the directive.
        pub fn value(&self) -> Cow<'static, str> {
            Cow::Borrowed(match self {
                CacheDirective::MaxAge(a) => return Cow::Owned(format!("max-age={a}")),
                CacheDirective::SMaxage(a) => return Cow::Owned(format!("s-maxage={a}")),
                CacheDirective::NoCache => "no-cache",
                CacheDirective::NoStore => "no-store",
                CacheDirective::NoTransform => "no-transform",
                CacheDirective::MustRevalidate => "must-revalidate",
                CacheDirective::ProxyRevalidate => "proxy-revalidate",
                CacheDirective::MustUnderstand => "must-understand",
                CacheDirective::Private => "private",
                CacheDirective::Public => "public",
                CacheDirective::Immutable => "immutable",
                CacheDirective::StaleWhileRevalidate(a) => {
                    return Cow::Owned(format!("stale-while-revalidate={a}"))
                }
                CacheDirective::StaleIfError(a) => {
                    return Cow::Owned(format!("stale-if-error={a}"))
                }
            })
        }
    }
}

mod connection {
    use super::*;

    ///`Connection` header for requests and responses. ([RFC 9110 §7.6.1](https://www.rfc-editor.org/rfc/rfc9110#field.connection))
    ///
    /// **This header is used internally by afire and should not be set manually**
    /// (unless you know what you're doing or something).
    ///
    /// Controls whether the network connection stays open after the current transaction finishes.
    /// If the value sent is keep-alive, the connection is persistent and not closed, allowing for subsequent requests to the same server to be done.
    ///
    /// [MDN Docs](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection)
    pub enum Connection {
        /// Indicates that either the client or the server would like to close the connection.
        /// This is the default behavior on HTTP/1.0 requests.
        Close,
        /// Indicates that the client would like to keep the connection open.
        /// This is the default behavior on HTTP/1.1 requests.
        ///
        /// The list contains the header names that are to be removed by the first non-transparent proxy or cache in-between the client and the server as they define the connection between the emitter and the first entity, not the destination node.
        /// A common example is the "keep-alive" header that defines the maximum number of requests that can be made through the same connection.
        Headers(Box<[HeaderName]>),
    }

    impl Connection {
        /// Creates a new Connection header with user-defined headers.
        pub fn new(headers: impl Into<Box<[HeaderName]>>) -> Self {
            Connection::Headers(headers.into())
        }

        /// Creates a new Connection header with the KeepAlive header.
        pub fn keep_alive() -> Self {
            Connection::Headers([HeaderName::KeepAlive].into())
        }

        /// Creates a new Connection header with the Close header.
        pub fn upgrade() -> Self {
            Connection::Headers([HeaderName::Upgrade].into())
        }
    }

    impl From<Connection> for Header {
        fn from(value: Connection) -> Self {
            let value = match value {
                Connection::Close => Cow::Borrowed("close"),
                Connection::Headers(h) => Cow::Owned(comma_separated(h, |x| x.to_string().into())),
            };

            Header {
                name: HeaderName::Connection,
                value,
            }
        }
    }
}

mod content_encoding {
    use super::*;

    /// `Content-Encoding` header for requests and responses. ([RFC 9110 §8.4](https://www.rfc-editor.org/rfc/rfc9110#field.content-encoding))
    ///
    /// Lists any encodings that have been applied to the message payload, and in what order.
    /// This lets the recipient know how to decode the representation in order to obtain the original payload format.
    ///
    /// Servers are encouraged to compress data as much as possible, and should use content encoding where appropriate.
    /// Compressing a compressed media type such as a zip or jpeg may not be appropriate, as this can make the payload larger.
    ///
    /// [MDN Docs](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Encoding)
    pub struct ContentEncoding {
        encodings: Box<[Encoding]>,
    }

    /// Encodings that can be used with `ContentEncoding`.
    pub enum Encoding {
        /// A format using the Lempel-Ziv coding (LZ77), with a 32-bit CRC.
        /// This is the original format of the UNIX gzip program.
        /// The HTTP/1.1 standard also recommends that the servers supporting this content-encoding should recognize x-gzip as an alias, for compatibility purposes.
        Gzip,
        /// A format using the Lempel-Ziv-Welch (LZW) algorithm.
        /// The value name was taken from the UNIX compress program, which implemented this algorithm.
        /// Like the compress program, which has disappeared from most UNIX distributions, this content-encoding is not used by many browsers today, partly because of a patent issue (it expired in 2003).
        Compress,
        /// Using the zlib structure (defined in RFC 1950) with the deflate compression algorithm (defined in RFC 1951).
        Deflate,
        /// A format using the Brotli algorithm.
        /// This format is not supported by all browsers.
        Br,
        /// A format using an algorithm not listed here.
        Other(Cow<'static, str>),
    }

    impl ContentEncoding {
        /// Create a new `ContentEncoding` header with the given encodings.
        pub fn new(encodings: impl Into<Box<[Encoding]>>) -> Self {
            ContentEncoding {
                encodings: encodings.into(),
            }
        }
    }

    impl From<ContentEncoding> for Header {
        fn from(value: ContentEncoding) -> Self {
            Header {
                name: HeaderName::ContentEncoding,
                value: Cow::Owned(comma_separated(value.encodings, |x| x.value())),
            }
        }
    }

    impl Encoding {
        /// Gets the text representation of the encoding.
        pub fn value(&self) -> Cow<'static, str> {
            Cow::Borrowed(match self {
                Encoding::Gzip => "gzip",
                Encoding::Compress => "compress",
                Encoding::Deflate => "deflate",
                Encoding::Br => "br",
                Encoding::Other(s) => return s.clone(),
            })
        }
    }
}

mod content_length {
    use super::*;

    /// `Content-Length` header for requests and responses. ([RFC 9110 §8.6](https://www.rfc-editor.org/rfc/rfc9110#field.content-length))
    ///
    /// Indicates the size of the message body, in bytes, sent to the recipient.
    ///
    /// [MDN Docs](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Length)
    pub struct ContentLength(u64);

    impl From<ContentLength> for Header {
        fn from(value: ContentLength) -> Self {
            Header {
                name: HeaderName::ContentLength,
                value: Cow::Owned(value.0.to_string()),
            }
        }
    }
}

mod content_type {
    use super::*;

    /// `Content-Type` header for requests and responses. ([RFC 9110 §8.3](https://www.rfc-editor.org/rfc/rfc9110#field.content-type))
    ///
    /// Indicates the original media type of the resource (prior to any content encoding applied for sending).
    /// On the client, this value can be ignored, for example when browsers perform MIME sniffing; set the X-Content-Type-Options header value to nosniff to prevent this behavior.
    ///
    /// [MDN Docs](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Type)
    pub struct ContentType {
        /// The MIME type of the resource or the data.
        mime: Mime,
        /// The character encoding standard.
        /// Case insensitive, lowercase is preferred.
        charset: Option<Charset>,
        /// For multipart entities the boundary directive is required.
        /// The directive consists of 1 to 70 characters from a set of characters (and not ending with white space) known to be very robust through email gateways.
        /// It is used to encapsulate the boundaries of the multiple parts of the message.
        boundary: Option<String>,
    }

    macro_rules! content_type_shortcut {
        [$(($name:ident, $mime:ident)),*] => {
            $(
                #[doc = concat!("Creates a ContentType header for ", stringify!($name), ", with the UTF-8 charset.")]
                pub fn $name() -> Self {
                    Self {
                        mime: mime::$mime,
                        charset: Some(Charset::Utf8),
                        boundary: None,
                    }
                }
            )*
        };
    }

    impl ContentType {
        /// Create a new ContentType header with the given MIME type.
        pub fn new(mime: impl Into<Mime>) -> Self {
            ContentType {
                mime: mime.into(),
                charset: None,
                boundary: None,
            }
        }

        content_type_shortcut![
            (html, HTML),
            (text, TEXT),
            (csv, CSV),
            (json, JSON),
            (xml, XML)
        ];
    }

    impl From<ContentType> for Header {
        fn from(value: ContentType) -> Self {
            let mut out = value.mime.to_string();
            if let Some(charset) = value.charset {
                out += &format!("; charset={}", charset);
            }
            if let Some(boundary) = value.boundary {
                out += &format!("; boundary={}", boundary);
            }

            Header {
                name: HeaderName::ContentType,
                value: Cow::Owned(out),
            }
        }
    }
}

mod date {
    use crate::{internal::misc::epoch, proto::http::date::imp_date};

    use super::*;

    /// `Date` header for requests and responses. ([RFC 9110 §6.6.1](https://www.rfc-editor.org/rfc/rfc9110#field.date))
    ///
    /// Contains the date and time at which the message originated.
    ///
    /// [MDN Docs](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Date)
    pub struct Date {
        time: u64,
    }

    impl Date {
        /// Create a new Date header with the current time.
        pub fn now() -> Self {
            Self {
                time: epoch().as_secs(),
            }
        }

        /// Create a new date header with the given time.c
        /// (Epoch time in seconds)
        pub fn epoch(epoch: u64) -> Self {
            Self { time: epoch }
        }
    }

    impl From<Date> for Header {
        fn from(value: Date) -> Self {
            Header {
                name: HeaderName::Date,
                value: Cow::Owned(imp_date(value.time)),
            }
        }
    }
}

mod location {
    use super::*;

    /// `Location` header for responses. ([RFC 9110 §10.2.2](https://www.rfc-editor.org/rfc/rfc9110#field.location))
    ///
    /// Indicates the URL to redirect a page to. It only provides a meaning when served with a 3xx (redirection) or 201 (created) status response.
    ///
    /// If you are working with redirects, you might want to use the [`crate::extensions::redirect::RedirectResponse`] extension.
    ///
    /// [MDN Docs](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Location)
    pub struct Location {
        url: Cow<'static, str>,
    }

    impl Location {
        /// Creates a new Location header with the given address.
        pub fn new(url: impl Into<Cow<'static, str>>) -> Self {
            Location { url: url.into() }
        }
    }

    impl From<Location> for Header {
        fn from(value: Location) -> Self {
            Header {
                name: HeaderName::Location,
                value: value.url,
            }
        }
    }
}

mod server {
    use super::*;

    /// `Server` header for responses. ([RFC 9110 §10.2.4](https://www.rfc-editor.org/rfc/rfc9110#field.server))
    pub struct Server {
        product: Cow<'static, str>,
    }

    impl Server {
        /// Create a new Server header with the given product name.
        pub fn new(product: impl Into<Cow<'static, str>>) -> Self {
            Server {
                product: product.into(),
            }
        }

        /// Create a new Server header with the afire product name and version.
        #[doc = concat!("Currently: `afire/", env!("CARGO_PKG_VERSION"), "`")]
        pub fn afire() -> Self {
            Server::new(concat!("afire/", env!("CARGO_PKG_VERSION")))
        }
    }

    impl From<Server> for Header {
        fn from(value: Server) -> Self {
            Header {
                name: HeaderName::Server,
                value: value.product,
            }
        }
    }
}

mod vary {
    use super::*;

    /// `Vary` header for responses. ([RFC 9110 §10.2.6](https://www.rfc-editor.org/rfc/rfc9110#field.vary))
    pub enum Vary {
        /// List of headers that may cause a cache to change to a response.
        Headers {
            /// The headers that may cause a cache to change to a response.
            headers: Box<[HeaderName]>,
        },
        /// Indicates that some non-header factor caused the response to change.
        Wildcard,
    }

    impl Vary {
        /// Create a new Vary header with the given header names.
        pub fn headers(headers: impl Into<Box<[HeaderName]>>) -> Self {
            Vary::Headers {
                headers: headers.into(),
            }
        }
    }

    impl From<Vary> for Header {
        fn from(value: Vary) -> Self {
            let value = match value {
                Vary::Headers { headers } => {
                    Cow::Owned(comma_separated(headers, |x| x.to_string().into()))
                }
                Vary::Wildcard => Cow::Borrowed("*"),
            };

            Header {
                name: HeaderName::Vary,
                value,
            }
        }
    }
}

mod misc {
    use super::*;

    /// A character encoding standard.
    pub enum Charset {
        /// UTF-8
        Utf8,
        /// A custom charset.
        Custom(String),
    }

    impl Display for Charset {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Charset::Utf8 => write!(f, "utf-8"),
                Charset::Custom(s) => write!(f, "{}", s),
            }
        }
    }

    impl From<&str> for Charset {
        fn from(value: &str) -> Self {
            match value.to_ascii_lowercase().as_str() {
                "utf-8" => Charset::Utf8,
                _ => Charset::Custom(value.to_string()),
            }
        }
    }
}

// todo: setcookie, TransferEncoding, Via, WWWAuthenticate

fn comma_separated<T>(
    values: impl Into<Box<[T]>>,
    to_string: fn(&T) -> Cow<'static, str>,
) -> String {
    values
        .into()
        .iter()
        .map(to_string)
        .collect::<Vec<_>>()
        .join(", ")
}
