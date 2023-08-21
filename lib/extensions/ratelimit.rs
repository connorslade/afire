//! An extension to limit the amount of requests sent from a single IP that will be handled by the server.

use std::collections::HashMap;
use std::fmt;
use std::net::IpAddr;
use std::sync::Arc;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    RwLock,
};

use crate::internal::misc::epoch;
use crate::Status;
use crate::{
    middleware::{MiddleResult, Middleware},
    Content, Request, Response,
};

// Handler Type
type Handler = Box<dyn Fn(&Request) -> Option<Response> + Send + Sync>;

/// Limit the amount of requests handled by the server.
pub struct RateLimiter {
    /// Requests Per Req_Timeout
    req_limit: u64,

    /// Time of last reset
    last_reset: AtomicU64,

    /// How often to reset the counters (sec)
    req_timeout: u64,

    /// Table that maps an IP to a list of request timestamps
    // requests: RwLock<HashMap<IpAddr, Vec<u64>>>,
    requests: RwLock<HashMap<IpAddr, u64>>,

    /// Handler for when the limit is reached.
    /// If the handler returns None, the request will be processed normally.
    handler: Handler,
}

impl RateLimiter {
    /// Make a new RateLimiter.
    ///
    /// Default limit is 10 and timeout is 60
    pub fn new() -> RateLimiter {
        RateLimiter {
            last_reset: AtomicU64::new(0),
            req_limit: 10,
            req_timeout: 60,
            requests: RwLock::new(HashMap::new()),
            handler: Box::new(|_| {
                Some(
                    Response::new()
                        .status(Status::TooManyRequests)
                        .text("Too Many Requests")
                        .content(Content::TXT),
                )
            }),
        }
    }

    /// Set the request limit per timeout
    /// Attach the rate limiter to a server.
    /// ## Example
    /// ```rust,no_run
    /// // Import Lib
    /// use afire::{Server, extension::RateLimiter, Middleware};
    ///
    /// // Create a new server
    /// let mut server = Server::<()>::new("localhost", 1234);
    ///
    /// // Add a rate limiter
    /// RateLimiter::new()
    ///     // Override limit to 100 requests
    ///     .limit(100)
    ///     // Attach it to the server
    ///     .attach(&mut server);
    ///
    /// // Start Server
    /// // This is blocking
    /// server.start().unwrap();
    /// ```
    pub fn limit(self, limit: u64) -> RateLimiter {
        RateLimiter {
            req_limit: limit,
            ..self
        }
    }

    /// Set the Ratelimit refresh period
    /// ## Example
    /// ```rust,no_run
    /// // Import Lib
    /// use afire::{Server, extension::RateLimiter, Middleware};
    ///
    /// // Create a new server
    /// let mut server = Server::<()>::new("localhost", 1234);
    ///
    /// // Add a rate limiter
    /// RateLimiter::new()
    ///     // Override timeout to 60 seconds
    ///     .timeout(60)
    ///     // Attach it to the server
    ///     .attach(&mut server);
    ///
    /// // Start Server
    /// // This is blocking
    /// server.start().unwrap();
    /// ```
    pub fn timeout(self, timeout: u64) -> RateLimiter {
        RateLimiter {
            req_timeout: timeout,
            ..self
        }
    }

    /// Define a Custom Handler for when a client has exceeded the ratelimit.
    /// If the handler returns None, the request will be processed normally.
    /// ## Example
    /// ```rust,no_run
    /// // Import Lib
    /// use afire::{Server, Response, extension::RateLimiter, Middleware};
    ///
    /// // Create a new server
    /// let mut server = Server::<()>::new("localhost", 1234);
    ///
    /// // Add a rate limiter
    /// RateLimiter::new()
    ///     // Override the handler for requests exceeding the limit
    ///     .handler(Box::new(|_req| Some(Response::new().text("much request"))))
    ///     // Attach it to the server
    ///     .attach(&mut server);
    ///
    /// // Start Server
    /// // This is blocking
    /// server.start().unwrap();
    /// ```
    pub fn handler(self, handler: Handler) -> RateLimiter {
        RateLimiter { handler, ..self }
    }

    /// Count a request.
    fn add_request(&self, ip: IpAddr) {
        let mut req = self.requests.write().unwrap();
        let count = req.get(&ip).unwrap_or(&0) + 1;
        req.insert(ip, count);
    }

    /// Check if request table needs to be cleared.
    fn check_reset(&self) {
        let time = epoch().as_secs();
        if self.last_reset.load(Ordering::Acquire) + self.req_timeout <= time {
            self.requests.write().unwrap().clear();
            self.last_reset.store(time, Ordering::Release);
        }
    }

    /// Check if the request limit has been reached for an ip.
    fn is_over_limit(&self, ip: IpAddr) -> bool {
        self.requests.read().unwrap().get(&ip).unwrap_or(&0) >= &self.req_limit
    }
}

impl Middleware for RateLimiter {
    fn pre(&self, req: &mut Request) -> MiddleResult {
        if self.is_over_limit(req.address.ip()) {
            if let Some(i) = (self.handler)(req) {
                return MiddleResult::Send(i);
            }
        }

        MiddleResult::Continue
    }

    fn end(&self, req: Arc<Request>, _res: &Response) {
        self.check_reset();
        self.add_request(req.address.ip());
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

// Allow printing of RateLimiter for debugging
impl fmt::Debug for RateLimiter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RateLimiter")
            .field("req_limit", &self.req_limit)
            .field("req_timeout", &self.req_timeout)
            .field("last_reset", &self.last_reset)
            .field("requests", &self.requests)
            .finish()
    }
}
