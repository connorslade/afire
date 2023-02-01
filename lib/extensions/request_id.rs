use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{
    error::Result,
    middleware::{MiddleRequest, Middleware},
    Header, Request,
};

/// Add an id to evey incomming Request
///
/// The is is just incremented on each request to not have to worry about collisions
pub struct RequestId {
    id_header: String,
    id: AtomicUsize,
}

impl RequestId {
    /// Create a new RequestId Middleware
    /// ## Example
    /// ```rust,no_run
    /// // Import Lib
    /// use afire::{Server, Middleware, extension::RequestId};
    ///
    /// // Create Server & RequestId Middleware
    /// let mut server = Server::<()>::new("localhost", 8080);
    /// RequestId::new("X-REQ-ID").attach(&mut server);
    ///
    /// // Start Server
    /// server.start().unwrap();
    ///```
    pub fn new<T: AsRef<str>>(header: T) -> Self {
        Self {
            id: AtomicUsize::new(0),
            id_header: header.as_ref().to_owned(),
        }
    }
}

impl Middleware for RequestId {
    fn pre(&self, req: &Result<Request>) -> MiddleRequest {
        let mut req = match req {
            Ok(req) => req.to_owned(),
            Err(_) => return MiddleRequest::Continue,
        };

        req.headers.insert(
            0,
            Header::new(
                &self.id_header,
                self.id.fetch_add(1, Ordering::Relaxed).to_string(),
            ),
        );

        MiddleRequest::Add(req)
    }
}
