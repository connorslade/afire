//! Cache responses in memory

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    error::Result,
    middleware::{MiddleRequest, Middleware},
    Method, Request, Response,
};

/// A Middleware to cache route responses
///
/// By defult it caches *every* request
/// this can be changed with the .to_cache method
pub struct Cache {
    /// Cache Data (Route -> (Route Response, Cache Epoch))
    cache: RwLock<HashMap<(Method, String), (Response, u64)>>,

    /// Function thaat defines weather a request should be cached
    to_cache: Box<dyn Fn(&Request) -> bool + Send + Sync>,

    /// Cache timeout
    ///
    /// To disable set to 0
    timeout: u64,
}

impl Cache {
    /// Create a new Cache middleware
    ///
    /// By defult it will cache every requests response
    /// even if it dosent go to a valid route.
    /// The timout is set to 3600 seconds or one hour by defult.
    /// ## Example
    /// ```rust
    /// // Import Stuff
    /// use afire::{Server, Middleware, extension::Cache};
    ///
    /// // Create Server
    /// let mut server = Server::<()>::new("localhost", 8080);
    /// // Add Cache Middleware
    /// Cache::new().attach(&mut server);
    /// ```
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            to_cache: Box::new(|_| true),
            timeout: 60 * 60,
        }
    }

    /// Set the function that defines weather a request will be cached
    ///
    /// The function must take in a Request refrence and return a bool
    /// True to cache and False to not
    ///
    /// Instead of writing your own function for this you can use some of the builtin ones at [`to_cache`]
    /// ## Example
    /// ```rust
    /// // Import Stuff
    /// use afire::{Server, Middleware, extension::Cache};
    ///
    /// // Create Server
    /// let mut server = Server::<()>::new("localhost", 8080);
    /// // Create and Add Cache Middleware
    /// Cache::new()
    ///     // Cache paths that start with `/cache`
    ///     .to_cache(|req| req.path.starts_with("/cache"))
    ///     .attach(&mut server);
    /// ```
    pub fn to_cache(self, fun: impl Fn(&Request) -> bool + 'static + Send + Sync) -> Self {
        Self {
            to_cache: Box::new(fun),
            ..self
        }
    }

    /// Set cache timeout in seconds
    /// When the cache expires the route will be run again to refresh response
    /// Set timeout to 0 do disable rerunning the route
    /// ## Example
    /// ```rust
    /// // Import Stuff
    /// use afire::{Server, Middleware, extension::Cache};
    ///
    /// // Create Server
    /// let mut server = Server::<()>::new("localhost", 8080);
    /// // Create and Add Cache Middleware
    /// Cache::new()
    ///     .timeout(60 * 60 * 24)
    ///     .attach(&mut server);
    /// ```
    pub fn timeout(self, timeout: u64) -> Self {
        Self { timeout, ..self }
    }
}

impl Middleware for Cache {
    fn pre(&self, req: &Result<Request>) -> MiddleRequest {
        let req = match req {
            Ok(i) => i.to_owned(),
            Err(_) => return MiddleRequest::Continue,
        };

        // Get response from cache
        if let Some((res, time)) = self
            .cache
            .read()
            .unwrap()
            .get(&(req.method.to_owned(), req.path.to_owned()))
        {
            // If resource has expired remove from cache and continue
            // Cache never times out if timeout is 0
            if self.timeout != 0 && current_epoch() - time >= self.timeout {
                self.cache
                    .write()
                    .unwrap()
                    .remove(&(req.method, req.path))
                    .unwrap();
                return MiddleRequest::Continue;
            }

            // Send cached response
            return MiddleRequest::Send(res.to_owned());
        }

        // No cached response found, continue
        MiddleRequest::Continue
    }

    fn end(&self, req: &Result<Request>, res: &Response) {
        let req = match req {
            Ok(i) => i.to_owned(),
            Err(_) => return,
        };

        // Return if its not ment to be
        // or if response is already in cache
        if !(self.to_cache)(&req)
            || self
                .cache
                .read()
                .unwrap()
                .get(&(req.method.to_owned(), req.path.to_owned()))
                .is_some()
        {
            return;
        }

        // Add resource to cache
        self.cache
            .write()
            .unwrap()
            .insert((req.method, req.path), (res.to_owned(), current_epoch()));
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
fn current_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards!?")
        .as_secs()
}

/// Builtin functions to define what paths are to be cached
/// ```no_test
/// Cache::new()
///     .to_cache(|x| fun_name(x, extra_prams))
///     .attach(&mut server);
/// ```
pub mod to_cache {
    use crate::{internal::path::Path, Request};

    /// Cache request paths that match with one of the paths supplied
    /// ```no_test
    /// Cache::new()
    ///     .to_cache(|x| path_match(x, &vec!["/cache/**"]))
    ///     .attach(&mut server);
    /// ```
    pub fn path_match(req: &Request, paths: &[&str]) -> bool {
        paths.iter().any(|x| {
            Path::new(req.path.clone())
                .match_path((*x).to_owned())
                .is_some()
        })
    }
}
