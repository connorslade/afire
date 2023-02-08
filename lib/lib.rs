#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

/// Current version of afire
#[doc(hidden)]
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Contains all the constants used in afire.
/// These may be in the future moved into the [`Server`] struct.
mod consts {
    /// The innitial buffer allocation for the request.
    pub const BUFF_SIZE: usize = 256;

    /// Max chunk size for chunked transfer encoding.
    pub const CHUNK_SIZE: usize = 16 * 1024;
}

// Export Internal Functions
pub mod internal;

// Import Internal Functions
mod thread_pool;
use http::*;
use internal::{encoding, handle, path};

#[macro_use]
pub mod trace;
pub mod error;
mod http;
pub mod middleware;
mod request;
mod response;
mod route;
mod server;
pub use self::{
    content_type::Content,
    cookie::{Cookie, SetCookie},
    error::Error,
    header::{Header, HeaderType},
    http::header,
    method::Method,
    middleware::Middleware,
    query::Query,
    request::Request,
    response::Response,
    route::Route,
    server::Server,
    status::Status,
};

/// The Prelude is a collection of very commenly used *things* in afire.
/// Unless you are using middleware, extentions or internial lower level stuff this should be all you need!
pub mod prelude {
    pub use crate::{
        error::{self, Error},
        middleware::{MiddleResult, Middleware},
        Content, Cookie, Header, HeaderType, Method, Request, Response, Server, SetCookie, Status,
    };
}

// Extra Features
#[cfg(feature = "extensions")]
mod extensions;
#[cfg(feature = "extensions")]
pub mod extension {
    //! Built in Extensions
    //!
    //! - Serve Static
    //! - RateLimit
    //! - Logger
    //! - Request Id
    //! - Date
    pub use crate::extensions::{
        date::Date,
        logger::{self, Logger},
        ratelimit::RateLimiter,
        request_id::RequestId,
        serve_static::{self, ServeStatic},
    };
}
