/*!
# 🔥 afire <a href="https://github.com/Basicprogrammer10/afire/actions"><img src="https://img.shields.io/github/workflow/status/Basicprogrammer10/afire/CI?label=Tests"></a> <a href="https://www.codefactor.io/repository/github/basicprogrammer10/watertemp"><a href="#"><img src="https://img.shields.io/tokei/lines/github/Basicprogrammer10/afire?label=Total%20Lines"></a> <a href="https://crates.io/crates/afire"><img src="https://img.shields.io/crates/d/afire"></a>
A blazing fast web framework for Rust

## 💠 Install

Just add the following to your `Cargo.toml`:
```toml
[dependencies]
afire = "0.1.0"
```

## 📄 Info
This is kinda like express.js for rust. It is not *that* complicated but it still makes development of apis / servers much easier. It supports Middleware and comes with some built in for Logging and Rate limiting.

For more information on this lib check the docs [here](https://crates.io/crates/afire)

## 💥 Examples

Make a simple server:
```rust
// Import Lib
use afire::*;

// Create Server
let mut server: Server = Server::new("localhost", 8080);

// Add a route
server.route(Method::GET, "/", |_req| {
    Response::new(
        200,
        "Hello World",
        vec![Header::new("Content-Type", "text/plain")],
    )
});

// Start the server
// This is blocking
# server.set_run(false);
server.start();
```
You can add as many routes as you want. The most recently defined route will always take priority. So you can make a 404 page like this:
```rust
// Import Library
use afire::{Server, Response, Header, Method};

// Create Server
let mut server: Server = Server::new("localhost", 8080);

// Define 404 page
// Because this is defined first, it will take a low priority
server.all(|req| {
    Response::new(
        404,
        "The page you are looking for does not exist :/",
        vec![Header::new("Content-Type", "text/plain")],
    )
});

// Define a route
// As this is defined last, it will take a high priority
server.route(Method::GET, "/hello", |req| {
    Response::new(
        200,
        "Hello World!",
        vec![Header::new("Content-Type", "text/plain")],
    )
});

// Starts the server
// This is blocking
# server.set_run(false);
server.start();
```

## 📦 Middleware
afire comes with some builtin extensions in the form of middleware.
### • ⛓️ Rate-limit
This will use the client ip to limit the amount of requests that will be processed. You can configure the request limit and the reset period.
```rust
// Import Stuff
use afire::{Server, RateLimiter};

// Make server
let mut server: Server = Server::new("localhost", 8080);

// Enable Rate Limiting
// This will limit the requests per ip to 5 every 10 sec
RateLimiter::attach(&mut server, 5, 10);
```
### • 📜 Logger
This will log all requests to a file or stdout or bolth. You can pick a log level that will determine if headers and body will be logged.
```rust
// Import Stuff
use afire::{Server, Logger, Level};

// Make server again
let mut server: Server = Server::new("localhost", 8080);

// Enable Logger
// Level::Debug has headers and body
// Level::Info does not
Logger::attach(
    &mut server,
    Logger::new(Level::Debug, Some("log.log"), true),
);
```
*/

mod common;

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

// Extra Features
mod extensions;

// Basic Rate Limiter
#[cfg(feature = "rate_limit")]
pub use extensions::RateLimiter;

#[cfg(feature = "logging")]
pub use extensions::{Level, Logger};
