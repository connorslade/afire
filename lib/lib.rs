#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

/// Current version of afire
#[doc(hidden)]
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Contains all the constants used in afire.
/// These may be in the future moved into the [`Server`] struct.
mod consts {
    /// The initial buffer allocation for the request.
    pub const BUFF_SIZE: usize = 256;

    /// Max chunk size for chunked transfer encoding.
    pub const CHUNK_SIZE: usize = 16 * 1024;
}

// Export Internal Functions
pub mod internal;

// Import Internal Functions
mod thread_pool;
use internal::{encoding, handle, router};
use proto::http::{self, *};

#[macro_use]
pub mod trace;
mod context;
pub mod error;
pub mod middleware;
mod proto;
mod request;
mod response;
pub mod route;
mod server;
mod socket;
pub use self::{
    content_type::Content,
    context::Context,
    cookie::{Cookie, SetCookie},
    error::Error,
    header::{Header, HeaderType},
    http::{cookie, header, multipart},
    method::Method,
    middleware::Middleware,
    proto::{server_sent_events, websocket},
    query::Query,
    request::Request,
    response::Response,
    server::Server,
    status::Status,
};

/// The Prelude is a collection of very commonly used *things* in afire.
/// Unless you are using middleware, extensions or internal lower level stuff this should be all you need!
pub mod prelude {
    pub use crate::{
        error::{self, Error},
        middleware::{MiddleResult, Middleware},
        proto::server_sent_events::ServerSentEventsExt,
        proto::websocket::WebSocketExt,
        Content, Cookie, Header, HeaderType, Method, Query, Request, Response, Server, SetCookie,
        Status,
    };
}

// Extra Features
#[cfg(feature = "extensions")]
#[path = "extensions/mod.rs"]
mod _extensions;

#[cfg(feature = "extensions")]
pub mod extensions {
    //! Useful extensions to the base afire.
    //! Includes helpful middleware like Serve Static, Rate Limit and Logger.
    //!
    //! ## All Feature
    //! | Name                 | Description                                           |
    //! | -------------------- | ----------------------------------------------------- |
    //! | [`Date`]             | Add the Date header to responses. Required by HTTP.   |
    //! | [`Head`]             | Add support for HTTP `HEAD` requests.                 |
    //! | [`Logger`]           | Log incoming requests to the console / file.          |
    //! | [`RateLimiter`]      | Limit how many requests can be handled from a source. |
    //! | [`RealIp`]           | Get the real IP of a client through a reverse proxy   |
    //! | [`RedirectResponse`] | Shorthands for HTTP redirects.                        |
    //! | [`RequestId`]        | Add a Request-Id header to all requests.              |
    //! | [`RouteShorthands`]  | Shorthands for defining routes (`server.get(...)`).   |
    //! | [`ServeStatic`]      | Serve static files from a dir.                        |
    //! | [`Trace`]            | Add support for the HTTP `TRACE` method.              |
    pub use crate::_extensions::{
        date::{self, Date},
        head::Head,
        logger::{self, Logger},
        ratelimit::RateLimiter,
        real_ip::RealIp,
        redirect::{self, RedirectResponse, RedirectType},
        request_id::RequestId,
        route_shorthands::RouteShorthands,
        serve_static::{self, ServeStatic},
        trace::Trace,
    };
}
