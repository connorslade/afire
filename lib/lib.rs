/*!
# 🔥 afire <a href="https://github.com/Basicprogrammer10/afire/actions"><img src="https://img.shields.io/github/workflow/status/Basicprogrammer10/afire/CI?label=Tests"></a> <a href="https://www.codefactor.io/repository/github/basicprogrammer10/watertemp"><a href="#"><img src="https://img.shields.io/tokei/lines/github/Basicprogrammer10/afire?label=Total%20Lines"></a> <a href="https://crates.io/crates/afire"><img src="https://img.shields.io/crates/d/afire"></a>
A blazing fast web framework for Rust

## 💠 Install

Just add the following to your `Cargo.toml`:
```toml
[dependencies]
afire = "0.1.0"
```

### TODO: 💥 Examples
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
