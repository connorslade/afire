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

pub mod internal;
mod thread_pool;
use internal::{encoding, handle, router};

#[macro_use]
pub mod trace;
mod context;
pub mod error;
pub mod middleware;
pub mod proto;
mod request;
mod response;
pub mod route;
mod server;
mod socket;

#[doc(inline)]
pub use self::{
    context::Context,
    error::Error,
    middleware::Middleware,
    proto::http::{
        content_type::Content,
        cookie,
        cookie::{Cookie, SetCookie},
        header,
        header::{Header, HeaderName},
        headers,
        method::Method,
        multipart,
        query::Query,
        status::Status,
    },
    proto::{server_sent_events, websocket},
    request::Request,
    response::Response,
    server::Server,
};

/// The prelude is a collection of types and traits that are commonly used in most afire applications.
pub mod prelude {
    pub use crate::{
        error::{self, Error},
        headers::{self, *},
        middleware::{MiddleResult, Middleware},
        proto::server_sent_events::ServerSentEventsExt,
        proto::websocket::WebSocketExt,
        route::{AdditionalRouteContext, RouteContext},
        Content, Context, Cookie, Header, HeaderName, Method, Query, Request, Response, Server,
        SetCookie, Status,
    };
}

#[cfg(feature = "extensions")]
pub mod extensions;
