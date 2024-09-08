//! Add an ID to every incoming Request in the form of a header.
//! The ID is just incremented on each request to not have to worry about collisions.

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{
    middleware::{MiddleResult, Middleware},
    HeaderName, Request,
};

/// Add an id to every incoming Request
///
/// The ID is just incremented on each request to not have to worry about collisions
pub struct RequestId {
    id_header: HeaderName,
    id: AtomicUsize,
}

impl RequestId {
    /// Create a new RequestId Middleware
    /// ## Example
    /// ```rust,no_run
    /// # use afire::{Server, Middleware, extensions::RequestId, error::Result};
    /// # fn main() -> Result<()> {
    /// // Create Server & RequestId Middleware
    /// let mut server = Server::builder("localhost", 8080, ()).build()?;
    /// RequestId::new("X-REQ-ID").attach(&mut server);
    ///
    /// // Start Server
    /// server.run()?;
    /// # Ok(())
    /// # }
    ///```
    pub fn new(header: impl Into<HeaderName>) -> Self {
        Self {
            id: AtomicUsize::new(0),
            id_header: header.into(),
        }
    }
}

impl Middleware for RequestId {
    fn pre(&self, req: &mut Request) -> MiddleResult {
        req.headers.add(
            &self.id_header,
            self.id.fetch_add(1, Ordering::Relaxed).to_string(),
        );

        MiddleResult::Continue
    }
}
