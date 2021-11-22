# 🔥 afire <a href="https://github.com/Basicprogrammer10/afire/actions"><img src="https://img.shields.io/github/workflow/status/Basicprogrammer10/afire/CI?label=Tests"></a> <a href="#"><img src="https://img.shields.io/tokei/lines/github/Basicprogrammer10/afire?label=Total%20Lines"></a> <a href="https://crates.io/crates/afire"><img src="https://img.shields.io/crates/d/afire?label=Downloads"></a>

A blazing fast dependency free web framework for Rust

## 💠 Install

Just add the following to your `Cargo.toml`:

```toml
[dependencies]
afire = "0.2.1"
```

## 📄 Info

This is kinda like express.js for rust. It is not _that_ complicated but it still makes development of apis / web servers much easier. It supports Middleware and comes with some built in for Static File Serving, Logging and Rate limiting.

For more information on this lib check the docs [here](https://crates.io/crates/afire)

## 💥 Examples

For some examples go [here](https://github.com/Basicprogrammer10/afire/tree/main/examples).

Here is a super simple examples:

```rust
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
afire = { version = "0.2.1", features = ["rate_limit", "logging", "serve_static"] }
```

- Threading

Just start the server like this. This will spawn a pool of threads to handle the requests. This is currently experimental and does not support middleware...

```rust
use afire::{Server, Method, Response, Header};

let mut server: Server = Server::new("localhost", 8080);

// server.start_threaded(8);
```
