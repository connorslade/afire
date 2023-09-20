use std::{
    borrow::Cow,
    fmt::{self, Display},
};

use crate::internal::misc::filter_crlf;

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
        pub enum HeaderName {
            $(
                $(#[$attr])*
                ///
                #[doc = concat!("[MDN Docs](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/", $header, ")")]
                $name
            ),*,
            /// Custom header type.
            /// Only used when the header type is unknown to afire.
            Custom(Cow<'static, str>),
        }

        impl HeaderName {
            fn from_str(s: &str) -> Self {
                use HeaderName::*;
                match s.to_ascii_lowercase().as_str() {
                    $($header_lower => $name),*,
                    _ => HeaderName::Custom(Cow::Owned(filter_crlf(s))),
                }
            }

            /// A custom header name.
            pub fn custom(s: impl Into<Cow<'static, str>>) -> Self {
                HeaderName::Custom(s.into())
            }

            /// Create a custom header name from a static string.
            pub const fn custom_str(s: &'static str) -> Self {
                HeaderName::Custom(Cow::Borrowed(s))
            }
        }

        impl Display for HeaderName {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                use HeaderName::*;
                f.write_str(match self {
                    $($name => $header),*,
                    HeaderName::Custom(c) => c,
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
    /// See [`HeaderName::AcceptEncoding`]
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
    /// Allows the sender of a HTTP message to hint about how the connection may be used and set a timeout and a maximum amount of requests.
    KeepAlive        => "Keep-Alive",        "keep-alive",
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

impl From<&HeaderName> for HeaderName {
    fn from(s: &HeaderName) -> Self {
        s.to_owned()
    }
}

impl<T: AsRef<str>> From<T> for HeaderName {
    fn from(s: T) -> Self {
        HeaderName::from_str(s.as_ref())
    }
}
