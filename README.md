# 🔥 afire <a href="https://github.com/Basicprogrammer10/afire/actions"><img src="https://img.shields.io/github/workflow/status/Basicprogrammer10/afire/CI?label=Tests"></a> <a href="#"><img src="https://img.shields.io/tokei/lines/github/Basicprogrammer10/afire?label=Total%20Lines"></a> <a href="https://crates.io/crates/afire"><img alt="Crates.io" src="https://img.shields.io/crates/v/afire"> <img src="https://img.shields.io/crates/d/afire?label=Downloads"></a>

A blazing fast dependency free web framework for Rust

## 💠 Install

Just add the following to your `Cargo.toml`:

```toml
[dependencies]
afire = "1.2.0"
```

## 📄 Info

afire is a _blazing fast_ web server micro framework for rust.
Its syntax is inspired by express.js.
It supports Middleware and comes with some built extensions in for Static File Serving, Rate limiting, more.

For more information on this lib check the docs [here](https://crates.io/crates/afire)

## 💥 Examples

For some examples go [here](https://github.com/Basicprogrammer10/afire/tree/main/examples).

Here is a super simple example:

```rust
// Import Lib
use afire::{Server, Method, Response, Header, Content};

// Create Server
let mut server = Server::new("localhost", 8080);

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

For these you will need to enable the `extension` feature like this:

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
let mut server = Server::new("localhost", 8080);

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

- Prelude

The Prelude is a collection of very commonly used _things_ in afire.
When using it you shouldn't need to import any other afire things unless you are using extensions or internal lower level stuff.
