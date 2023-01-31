#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

/// Current version of afire
#[doc(hidden)]
pub const VERSION: &str = "1.3.0*";
/// The defualt buffer allocation
const BUFF_SIZE: usize = 256;

// Export Internal Functions
#[macro_use]
pub mod internal;

// Import Internal Functions
mod thread_pool;
use internal::{common, handle, path};

// The main server
mod server;
pub use self::server::Server;

// HTTP Header relates things
mod header;
pub use self::header::Header;

// Different types of requests e.g. GET, POST, PUT, DELETE
mod method;
pub use self::method::Method;

// Routing - the main way of getting things done
mod route;
pub use self::route::Route;

// A request object to hold all the information about a request
mod request;
pub use self::request::Request;

// A response object that is used to define data to send to the client
mod response;
pub use self::response::Response;

// Query string stuff
mod query;
pub use self::query::Query;

// Content Types
mod content_type;
pub use content_type::Content;

// Middleware and stuff
pub mod middleware;
pub use middleware::Middleware;

// Errors from parseing and handling requests
pub mod error;
pub use error::Error;

// Cookies 🍪
mod cookie;
pub use self::cookie::{Cookie, SetCookie};

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
