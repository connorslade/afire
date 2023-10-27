//! Useful extensions to the base afire.
//! Includes helpful middleware like Serve Static, Rate Limit and Logger.
//!
//! ## All Feature
//! | Name                 | Description                                                               |
//! | -------------------- | ------------------------------------------------------------------------- |
//! | [`Date`]             | Add the Date header to responses. Required by HTTP.                       |
//! | [`Head`]             | Add support for HTTP `HEAD` requests.                                     |
//! | [`Logger`]           | Log incoming requests to the console / file.                              |
//! | [`PathNormalizer`]   | Normalize paths to a common format without trailing or repeating slashes. |
//! | [`RateLimiter`]      | Limit how many requests can be handled from a source.                     |
//! | [`RealIp`]           | Get the real IP of a client through a reverse proxy                       |
//! | [`RedirectResponse`] | Shorthands for HTTP redirects.                                            |
//! | [`RequestId`]        | Add a Request-Id header to all requests.                                  |
//! | [`RouteShorthands`]  | Shorthands for defining routes (`server.get(...)`).                       |
//! | [`ServeStatic`]      | Serve static files from a dir.                                            |
//! | [`SyncRoute`]        | Lets you make routes that return responses synchronously.                 |
//! | [`Trace`]            | Add support for the HTTP `TRACE` method.                                  |

pub mod date;
pub mod head;
pub mod logger;
pub mod path_normalizer;
pub mod range;
pub mod ratelimit;
pub mod real_ip;
pub mod redirect;
pub mod request_id;
pub mod route_shorthands;
pub mod serve_static;
pub mod sync_route;
pub mod trace;

#[doc(inline)]
pub use self::{
    date::Date,
    head::Head,
    logger::Logger,
    path_normalizer::PathNormalizer,
    range::Range,
    ratelimit::RateLimiter,
    real_ip::RealIp,
    redirect::{RedirectResponse, RedirectType},
    request_id::RequestId,
    route_shorthands::RouteShorthands,
    serve_static::ServeStatic,
    sync_route::SyncRoute,
    trace::Trace,
};
