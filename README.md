# 🔥 afire <a href="https://github.com/Basicprogrammer10/afire/actions"><img src="https://img.shields.io/github/workflow/status/Basicprogrammer10/afire/CI?label=Tests"></a> <a href="#"><img src="https://img.shields.io/tokei/lines/github/Basicprogrammer10/afire?label=Total%20Lines"></a> <a href="https://crates.io/crates/afire"><img src="https://img.shields.io/crates/d/afire?label=Downloads"></a>

A blazing fast dependency free web framework for Rust

## 💠 Install

Just add the following to your `Cargo.toml`:

```toml
[dependencies]
afire = "1.0.0"
```

## 📄 Info

This is kinda like express.js for rust. It is not _that_ complicated but it still makes development of apis / web servers much easier. It supports Middleware and comes with some built in for Static File Serving, Logging and Rate limiting.

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
server.start().unwrap();
```

## 🔧 Features

Here I will outline interesting features that are available in afire.

- Builtin Middleware

afire comes with some builtin extensions in the form of middleware.
Currently the builtin middleware includes [rate_limit](), [logging](), and [serve_static]().
For these you will need to enable the features.

To use these extra features enable them like this:

```toml
afire = { version = "1.0.0", features = ["rate_limit", "logging", "serve_static"] }
```

- Content Types

As an easy way to set the Content-Type of a Response you can use the `.content` methood of the Response.
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
server.start().unwrap();
```
/// Prelude
