/*!
# 🔥 afire <a href="https://github.com/Basicprogrammer10/afire/actions"><img src="https://img.shields.io/github/workflow/status/Basicprogrammer10/afire/CI?label=Tests"></a> <a href="#"><img src="https://img.shields.io/tokei/lines/github/Basicprogrammer10/afire?label=Total%20Lines"></a> <a href="https://crates.io/crates/afire"><img alt="Crates.io" src="https://img.shields.io/crates/v/afire"> <img src="https://img.shields.io/crates/d/afire?label=Downloads"></a>

A blazing fast dependency free web framework for Rust

## 💠 Install

Just add the following to your `Cargo.toml`:

```toml
[dependencies]
afire = "1.2.0"
```

## 📄 Info

This is kinda like express.js for rust. It is not _that_ complicated but it still makes development of APIs / web servers much easier. It supports Middleware and comes with some built in for Static File Serving, Rate limiting,  more.

For more information on this lib check the docs [here](https://crates.io/crates/afire)

## 💥 Examples

For some examples go [here](https://github.com/Basicprogrammer10/afire/tree/main/examples).

Here is a super simple example:

```rust
// Import Lib
use afire::{Server, Method, Response, Header, Content};

// Create Server
let mut server: Server = Server::new("localhost", 8080);

// Add a route
server.route(Method::GET, "/greet/{name}", |req| {
  let name = req.path_param("name").unwrap();

  Response::new()
    .text(format!("Hello, {}", name))
    .content(Content::TXT)
});

// Start the server
// This is blocking
# server.set_run(false);
server.start().unwrap();
```

## 🔧 Features

Here I will outline interesting features that are available in afire.

- Builtin Middleware

afire comes with some built-in extensions in the form of middleware.
Currently, the built-in middleware includes the following:
- Serve Static
- RateLimit
- Logger
- Response Cache
- Request ID

For these you will need to enable the features.

To use these extra features enable them like this:

```toml
afire = { version = "1.2.0", features = ["extension"] }
```

- Content Types

As an easy way to set the Content-Type of a Response you can use the `.content` method of the Response.
Then you can put one of the common predefined types.

```rust
// Import Lib
use afire::{Server, Method, Response, Header, Content};

// Create Server
let mut server: Server = Server::new("localhost", 8080);

// Add a route
server.route(Method::GET, "/", |_req| {
  Response::new()
    .text("Hello, World!")
    .content(Content::TXT)
});

// Start the server
// This is blocking
# server.set_run(false);
server.start().unwrap();
```
*/

// #![feature(test)]
#![warn(missing_docs)]

#[doc(hidden)]
pub const VERSION: &str = "1.1.0*";

// Export Internal Functions
#[macro_use]
pub mod internal;

// Import Internal Functions
mod handle;
mod thread_pool;
use internal::common;
use internal::http;
use internal::path;

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

// Cookies 🍪
#[cfg(feature = "cookies")]
mod cookie;
#[cfg(feature = "cookies")]
pub use self::cookie::{Cookie, SetCookie};

/// The Prelude is a collection of very commenly used *things* in afire
/// Unless you are using extentions or internial lower level stuff
pub mod prelude {
    pub use crate::{
        middleware::{HandleError, MiddleRequest, MiddleResponse, Middleware},
        Content, Header, Method, Request, Response, Server,
    };
    #[cfg(feature = "cookies")]
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
        cache::{self, Cache},
        logger::{self, Logger},
        ratelimit::RateLimiter,
        request_id::RequestId,
        serve_static::{self, ServeStatic},
    };
}

// Unit Tests
#[cfg(test)]
mod tests;
