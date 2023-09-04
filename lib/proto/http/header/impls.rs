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
pub use location::*;
pub use misc::*;

mod access_control_allow_origin {
    use crate::internal::misc::filter_crlf;

    use super::*;

    pub enum AccessControlAllowOrigin {
        Any,
        Null,
        Origin(Cow<'static, str>),
    }

    impl AccessControlAllowOrigin {
        pub fn origin(origin: impl Into<Cow<'static, str>>) -> Self {
            AccessControlAllowOrigin::Origin(origin.into())
        }
    }

    impl Into<Header> for AccessControlAllowOrigin {
        fn into(self) -> Header {
            let value = match self {
                AccessControlAllowOrigin::Any => "*".to_string(),
                AccessControlAllowOrigin::Null => "null".to_string(),
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

    pub struct CacheControl {
        directives: Box<[CacheDirective]>,
    }

    /// [MDN Docs](https://developer.mozilla.org/en-US/docs/web/http/headers/Cache-Control#cache_directives)
    pub enum CacheDirective {
        /// "max-age"
        MaxAge(u32),
        /// "s-maxage"
        SMaxage(u32),
        /// "no-cache"
        NoCache,
        /// "no-store"
        NoStore,
        /// "no-transform"
        NoTransform,
        /// "must-revalidate"
        MustRevalidate,
        /// "proxy-revalidate"
        ProxyRevalidate,
        /// "must-understand"
        MustUnderstand,
        /// "private"
        Private,
        /// "public"
        Public,
        /// "immutable"
        Immutable,
        /// "stale-while-revalidate"
        StaleWhileRevalidate,
        /// "stale-if-error"
        StaleIfError,
    }

    impl CacheControl {
        pub fn new(directives: impl Into<Box<[CacheDirective]>>) -> Self {
            CacheControl {
                directives: directives.into(),
            }
        }

        pub fn max_age(age: u32) -> Self {
            CacheControl::new([CacheDirective::MaxAge(age)])
        }

        pub fn no_cache() -> Self {
            CacheControl::new([CacheDirective::NoCache])
        }
    }

    impl Into<Header> for CacheControl {
        fn into(self) -> Header {
            let out = self
                .directives
                .iter()
                .fold(String::new(), |mut acc, directive| {
                    acc += &format!(", {}", directive.value());
                    acc
                });

            Header {
                name: HeaderName::CacheControl,
                value: Cow::Owned(out),
            }
        }
    }

    impl CacheDirective {
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
                CacheDirective::StaleWhileRevalidate => "stale-while-revalidate",
                CacheDirective::StaleIfError => "stale-if-error",
            })
        }
    }
}

mod connection {
    use super::*;

    pub enum Connection {
        KeepAlive,
        Close,
        Upgrade,
    }

    impl Into<Header> for Connection {
        fn into(self) -> Header {
            let value = match self {
                Connection::KeepAlive => "keep-alive",
                Connection::Close => "close",
                Connection::Upgrade => "upgrade",
            };

            Header {
                name: HeaderName::Connection,
                value: Cow::Borrowed(value),
            }
        }
    }
}

mod content_encoding {
    use super::*;

    pub struct ContentEncoding {
        encodings: Box<[Encoding]>,
    }

    pub enum Encoding {
        Gzip,
        Compress,
        Deflate,
        Br,
        Other(Cow<'static, str>),
    }

    impl ContentEncoding {
        pub fn new(encodings: impl Into<Box<[Encoding]>>) -> Self {
            ContentEncoding {
                encodings: encodings.into(),
            }
        }
    }

    impl Into<Header> for ContentEncoding {
        fn into(self) -> Header {
            let out = self
                .encodings
                .iter()
                .fold(String::new(), |mut acc, encoding| {
                    acc += &format!(", {}", encoding.value());
                    acc
                });

            Header {
                name: HeaderName::ContentEncoding,
                value: Cow::Owned(out),
            }
        }
    }

    impl Encoding {
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

    pub struct ContentLength(u64);

    impl Into<Header> for ContentLength {
        fn into(self) -> Header {
            Header {
                name: HeaderName::ContentLength,
                value: Cow::Owned(self.0.to_string()),
            }
        }
    }
}

mod content_type {
    use super::*;

    pub struct ContentType {
        mime: Mime,
        charset: Option<Charset>,
        boundary: Option<String>,
    }

    impl ContentType {
        pub fn new(mime: impl Into<Mime>) -> Self {
            ContentType {
                mime: mime.into(),
                charset: None,
                boundary: None,
            }
        }

        pub fn text() -> Self {
            ContentType {
                mime: mime::TEXT,
                charset: Some(Charset::Utf8),
                boundary: None,
            }
        }
    }

    impl Into<Header> for ContentType {
        fn into(self) -> Header {
            let mut out = self.mime.to_string();
            if let Some(charset) = self.charset {
                out += &format!("; charset={}", charset);
            }
            if let Some(boundary) = self.boundary {
                out += &format!("; boundary={}", boundary);
            }

            Header::new(HeaderName::ContentType, out)
        }
    }
}

mod date {
    use crate::{internal::misc::epoch, proto::http::date::imp_date};

    use super::*;

    pub struct Date {
        time: u64,
    }

    impl Date {
        pub fn now() -> Self {
            Self {
                time: epoch().as_secs(),
            }
        }

        pub fn epoch(epoch: u64) -> Self {
            Self { time: epoch }
        }
    }

    impl Into<Header> for Date {
        fn into(self) -> Header {
            Header {
                name: HeaderName::Date,
                value: Cow::Owned(imp_date(self.time)),
            }
        }
    }
}

mod location {
    use super::*;

    pub struct Location {
        url: Cow<'static, str>,
    }

    impl Location {
        pub fn new(url: impl Into<Cow<'static, str>>) -> Self {
            Location { url: url.into() }
        }
    }

    impl Into<Header> for Location {
        fn into(self) -> Header {
            Header {
                name: HeaderName::Location,
                value: self.url,
            }
        }
    }
}

mod server {
    use super::*;

    pub struct Server {
        product: Cow<'static, str>,
    }

    impl Server {
        pub fn new(product: impl Into<Cow<'static, str>>) -> Self {
            Server {
                product: product.into(),
            }
        }

        pub fn afire() -> Self {
            Server::new(concat!("afire/", env!("CARGO_PKG_VERSION")))
        }
    }
}

mod misc {
    use std::str::FromStr;

    use super::*;

    pub enum Charset {
        Utf8,
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
