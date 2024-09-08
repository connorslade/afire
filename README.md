# 🔥 afire <a href="https://github.com/connorslade/afire/actions"><img src="https://github.com/connorslade/afire/actions/workflows/rust.yml/badge.svg"></a> <a href="https://crates.io/crates/afire"><img alt="Crates.io" src="https://img.shields.io/crates/v/afire"></a> <a href="https://crates.io/crates/afire"><img src="https://img.shields.io/crates/d/afire?label=Downloads"></a>

# **THIS IS AN ALPHA RELEASE FOR `v3.0.0`** &ndash; Its probably not the best idea to use this in production and it will definitely have a lot of breaking changes in the future

afire is a _blazingly fast_ web server micro framework for rust.

## 💠 Install

Just add the following to your `Cargo.toml`:

```toml
[dependencies]
afire = "3.0.0-alpha.3"
```

## 📄 Info

afire is a simple synchronous multithreaded express.js inspired rust web micro framework.
wow that was long.
It comes with some built extensions in for Static File Serving, Rate limiting, and more.

Below you can find links to some afire related resources.

- [Homepage](https://connorcode.com/writing/afire)
- [Crates.io page](https://crates.io/crates/afire)
- [API Docs](https://docs.rs/afire/latest/afire)

## 💥 Example

For more examples see the examples directory [here](https://github.com/connorslade/afire/tree/main/examples).

Below is a super simple example so you can see the basics of afire syntax.

```rust ignore
use afire::prelude::*;

let mut server = Server::builder("localhost", 8080, ()).workers(16).build()?;

server.route(Method::GET, "/greet/{name}", |ctx| {
  let name = ctx.param("name");

  ctx.text(format!("Hello, {}", name))
      .content(Content::TXT)
      .send()?;

  Ok(())
});

server.run()?;
```

## 💼 License

afire is licensed under the MIT license so you are free to do basically whatever you want with it as long as you add a copyright notice.
You can read the full license text [here](https://github.com/connorslade/afire/blob/main/LICENSE).
