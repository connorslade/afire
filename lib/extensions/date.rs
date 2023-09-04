//! Middleware to add the HTTP Date header (as defined in [RFC 9110, Section 5.6.7](https://www.rfc-editor.org/rfc/rfc9110.html#section-5.6.7)).
//! This is technically required for all servers that have a clock, so I may move it to the core library at some point.

use crate::{
    internal::misc::epoch,
    middleware::{MiddleResult, Middleware},
    proto::http::date::imp_date,
    HeaderName, Request, Response,
};

/// Middleware to add the HTTP Date header (as defined in [RFC 9110, Section 5.6.7](https://www.rfc-editor.org/rfc/rfc9110.html#section-5.6.7)).
/// This is technically required for all servers that have a clock, so I may move it to the core library at some point.
///
/// ## Example
/// ```rust
/// # use afire::{extensions::Date, Middleware};
/// # fn add(mut server: afire::Server) {
/// Date.attach(&mut server);
/// # }
pub struct Date;

impl Middleware for Date {
    fn post(&self, _req: &Request, res: &mut Response) -> MiddleResult {
        let epoch = epoch().as_secs();
        res.headers.add(HeaderName::Date, imp_date(epoch));
        MiddleResult::Continue
    }
}
