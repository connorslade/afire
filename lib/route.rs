use super::method::Method;
use super::request::Request;
use super::response::Response;

/// Defines a route.
///
/// You should not use this directly.
/// It will be created automatically when useing server.get / post / put / delete / etc.
pub struct Route {
    pub(super) method: Method,
    pub(super) path: String,
    pub(super) handler: fn(Request) -> Response,
}
