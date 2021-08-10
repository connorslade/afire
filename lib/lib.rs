/*!
# 🔥 afire <a href="https://github.com/Basicprogrammer10/afire/actions"><img src="https://img.shields.io/github/workflow/status/Basicprogrammer10/afire/CI?label=Tests"></a> <a href="https://www.codefactor.io/repository/github/basicprogrammer10/watertemp"><a href="#"><img src="https://img.shields.io/tokei/lines/github/Basicprogrammer10/afire?label=Total%20Lines"></a>
A blazing fast web framework for Rust

Work in progress :P
*/

// HTTP Header relates things
// Thare is a lot :P
mod header;
pub use self::header::Header;

// The main server
mod server;
pub use self::server::Server;

// Other things like Methods, Routes, Request and Response defanitions
// Too small to need its own file
mod other;
pub use self::other::{Method, Request, Response, Route};
