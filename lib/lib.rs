/*!
# 🔥 afire <a href="https://github.com/Basicprogrammer10/afire/actions"><img src="https://img.shields.io/github/workflow/status/Basicprogrammer10/afire/CI?label=Tests"></a> <a href="#"><img src="https://img.shields.io/tokei/lines/github/Basicprogrammer10/afire?label=Total%20Lines"></a> <a href="https://crates.io/crates/afire"><img src="https://img.shields.io/crates/d/afire?label=Downloads"></a>

A blazing fast dependency free web framework for Rust

## 💠 Install

Just add the following to your `Cargo.toml`:

```toml
[dependencies]
afire = "0.2.1"
```

## 📄 Info

This is kinda like express.js for rust. It is not _that_ complicated but it still makes development of apis / servers much easier. It supports Middleware and comes with some built in for Logging and Rate limiting.

For more information on this lib check the docs [here](https://crates.io/crates/afire)

## 💥 Examples

For some examples go [here](https://github.com/Basicprogrammer10/afire/tree/main/examples).

Here is a super simple examples:

```no_run
// Import Lib
use afire::{Server, Method, Response, Header};

// Create Server
let mut server: Server = Server::new("localhost", 8080);

// Add a route
server.route(Method::GET, "/", |_req| {
  Response::new()
    .status(200)
    .text("Hello World!")
    .header(Header::new("Content-Type", "text/plain"))
});

// Start the server
// This is blocking

server.start().unwrap();

// Or use multi threading *experimental*
// server.start_threaded(8);
```

## 🔧 Features

Here I will outline interesting features that are available in afire.

- Builtin Middleware

afire comes with some builtin extensions in the form of middleware.
For these you will need to enable the feature.

To use these extra features enable them like this:

```toml
afire = { version = "0.2.1", features = ["rate_limit", "logging"] }
```

- Threading

Just start the server like this. This will spawn a pool of threads to handle the requests. This is currently experimental and does not support middleware...

```rust
use afire::{Server, Method, Response, Header};

let mut server: Server = Server::new("localhost", 8080);

# server.set_run(false);
// server.start_threaded(8);
```
*/

#![warn(missing_docs)]

pub(crate) const VERSION: &str = "0.2.1*";

mod common;
mod http;
// #[cfg(feature = "thread_pool")]
// mod threadpool;

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

// Cookies 🍪
#[cfg(feature = "cookies")]
mod cookie;
#[cfg(feature = "cookies")]
pub use self::cookie::{Cookie, SetCookie};

// Extra Features
mod extensions;

#[cfg(feature = "rate_limit")]
pub use extensions::ratelimit::RateLimiter;

#[cfg(feature = "logging")]
pub use extensions::logger::{Level, Logger};

#[cfg(feature = "serve_static")]
pub use extensions::serve_static::ServeStatic;
