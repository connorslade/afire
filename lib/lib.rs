#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

/// Current version of afire
#[doc(hidden)]
pub const VERSION: &str = "1.3.0*";

mod consts {
    /// The default buffer allocation
    pub const BUFF_SIZE: usize = 256;

    /// Max chunk size
    pub const CHUNK_SIZE: usize = 16 * 1024;
}

// Export Internal Functions
pub mod internal;

// Import Internal Functions
mod thread_pool;
use internal::{common, handle, path};

#[macro_use]
pub mod trace;
mod content_type;
mod cookie;
pub mod error;
mod header;
mod method;
pub mod middleware;
mod query;
mod request;
mod response;
mod route;
mod server;
pub use self::{
    content_type::Content,
    cookie::{Cookie, SetCookie},
    error::Error,
    header::Header,
    method::Method,
    middleware::Middleware,
    query::Query,
    request::Request,
    response::Response,
    route::Route,
    server::Server,
};

/// The Prelude is a collection of very commenly used *things* in afire
/// Unless you are using extentions or internial lower level stuff
pub mod prelude {
    pub use crate::{
        error::{self, Error},
        middleware::{MiddleRequest, MiddleResponse, Middleware},
        Content, Header, Method, Request, Response, Server,
    };
    pub use crate::{Cookie, SetCookie};
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
    //! - Response Cache
    //! - Request Id
    pub use crate::extensions::{
        // cache::{self, Cache},
        logger::{self, Logger},
        ratelimit::RateLimiter,
        request_id::RequestId,
        serve_static::{self, ServeStatic},
    };
}

// Unit Tests
#[cfg(test)]
mod tests;
