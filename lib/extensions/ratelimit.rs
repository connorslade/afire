use std::collections::HashMap;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::common::remove_address_port;
use crate::middleware::{MiddleRequest, Middleware};
use crate::{Request, Response};

// Handler Type
type Handler = Box<dyn Fn(&Request) -> Option<Response>>;

/// Limit the amount of requests handled by the server.
pub struct RateLimiter {
    /// Requests Per Req_Timeout
    req_limit: u64,

    /// Time of last reset
    last_reset: u64,

    /// How often to reset the counters (sec)
    req_timeout: u64,

    /// Table of requests per IP
    requests: HashMap<String, u64>,

    /// Handler for when the limit is reached
    handler: Handler,
}

impl RateLimiter {
    /// Make a new RateLimiter.
    ///
    /// Default limit is 10 and timeout is 60
    pub fn new() -> RateLimiter {
        RateLimiter {
            last_reset: 0,
            req_limit: 10,
            req_timeout: 60,
            requests: HashMap::new(),
            handler: Box::new(|_| {
                Some(
                    Response::new()
                        .status(429)
                        .text("Too Many Requests")
                        .header("Content-Type", "text/plain"),
                )
            }),
        }
    }

    /// Set the request limit per timeout
    /// Attach the rate limiter to a server.
    /// ## Example
    /// ```rust
    /// // Import Lib
    /// use afire::{Server, RateLimiter, Middleware};
    ///
    /// // Create a new server
    /// let mut server: Server = Server::new("localhost", 1234);
    ///
    /// // Add a rate limiter
    /// RateLimiter::new()
    ///     // Overide limit to 100 requests
    ///     .limit(100)
    ///     // Attatch it to the server
    ///     .attach(&mut server);
    ///
    /// // Start Server
    /// // This is blocking
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn limit(self, limit: u64) -> RateLimiter {
        RateLimiter {
            req_limit: limit,
            ..self
        }
    }

    /// Set the Ratelimit refresh peroid
    /// ## Example
    /// ```rust
    /// // Import Lib
    /// use afire::{Server, RateLimiter, Middleware};
    ///
    /// // Create a new server
    /// let mut server: Server = Server::new("localhost", 1234);
    ///
    /// // Add a rate limiter
    /// RateLimiter::new()
    ///     // Overide timeout to 60 seconds
    ///     .timeout(60)
    ///     // Attatch it to the server
    ///     .attach(&mut server);
    ///
    /// // Start Server
    /// // This is blocking
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn timeout(self, timeout: u64) -> RateLimiter {
        RateLimiter {
            req_timeout: timeout,
            ..self
        }
    }

    /// Define a Custom Handler for when a client has exceded the ratelimit
    /// ## Example
    /// ```rust
    /// // Import Lib
    /// use afire::{Server, Response, RateLimiter, Middleware};
    ///
    /// // Create a new server
    /// let mut server: Server = Server::new("localhost", 1234);
    ///
    /// // Add a rate limiter
    /// RateLimiter::new()
    ///     // Overide the handler for requests exceding the limit
    ///     .handler(Box::new(|_req| Some(Response::new().text("much request"))))
    ///     // Attatch it to the server
    ///     .attach(&mut server);
    ///
    /// // Start Server
    /// // This is blocking
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn handler(self, handler: Handler) -> RateLimiter {
        RateLimiter { handler, ..self }
    }

    /// Count a request.
    fn add_request(&mut self, ip: String) {
        self.requests
            .insert(ip.clone(), self.requests.get(&ip).unwrap_or(&0) + 1);
    }

    /// Check if request table needs to be cleared.
    fn check_reset(&mut self) {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if self.last_reset + self.req_timeout <= time {
            self.requests = HashMap::new();
            self.last_reset = time;
        }
    }

    /// Check if the request limit has been reached for an ip.
    fn is_over_limit(&self, ip: String) -> bool {
        self.requests.get(&ip).unwrap_or(&0) >= &self.req_limit
    }
}

impl Middleware for RateLimiter {
    fn pre(&mut self, req: Request) -> MiddleRequest {
        let ip = remove_address_port(&req.address);

        self.check_reset();

        if self.is_over_limit(ip.clone()) {
            return match (self.handler)(&req) {
                Some(i) => MiddleRequest::Send(i),
                None => MiddleRequest::Continue,
            };
        }

        self.add_request(ip);

        MiddleRequest::Continue
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
