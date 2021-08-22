# 🔥 afire <a href="https://github.com/Basicprogrammer10/afire/actions"><img src="https://img.shields.io/github/workflow/status/Basicprogrammer10/afire/CI?label=Tests"></a> <a href="https://www.codefactor.io/repository/github/basicprogrammer10/watertemp"><a href="#"><img src="https://img.shields.io/tokei/lines/github/Basicprogrammer10/afire?label=Total%20Lines"></a> <a href="https://crates.io/crates/afire"><img src="https://img.shields.io/crates/d/afire"></a>
<a href="https://app.fossa.com/projects/git%2Bgithub.com%2FBasicprogrammer10%2Fafire?ref=badge_shield" alt="FOSSA Status"><img src="https://app.fossa.com/api/projects/git%2Bgithub.com%2FBasicprogrammer10%2Fafire.svg?type=shield"/></a>
A blazing fast web framework for Rust

## 💠 Install

Just add the following to your `Cargo.toml`:
```toml
[dependencies]
afire = "0.1.0"
```

## 📄 Info
This is kinda like express.js for rust. It is not *that* complicated but it still makes development of apis / servers much easier.

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
server.start();
```

## License
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FBasicprogrammer10%2Fafire.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2FBasicprogrammer10%2Fafire?ref=badge_large)