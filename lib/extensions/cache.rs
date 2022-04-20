//! Cache responses in memory

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    middleware::{MiddleRequest, Middleware},
    Request, Response,
};

pub struct Cache {
    cache: RwLock<HashMap<String, (Response, u64)>>,

    /// Function thaat defines weather a request should be cached
    to_cache: Box<dyn Fn(&Request) -> bool + Send + Sync>,
    timeout: u64,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            to_cache: Box::new(|_| true),
            timeout: 60 * 60,
        }
    }

    pub fn to_cache(self, fun: impl Fn(&Request) -> bool + 'static + Send + Sync) -> Self {
        Self {
            to_cache: Box::new(fun),
            ..self
        }
    }
}

impl Middleware for Cache {
    fn pre(&self, req: &Request) -> MiddleRequest {
        // Get response from cache
        if let Some((res, time)) = self.cache.read().unwrap().get(&req.path) {
            // If resource has expired remove from cache and continue
            if current_epoch() - time >= self.timeout {
                self.cache.write().unwrap().remove(&req.path).unwrap();
                return MiddleRequest::Continue;
            }

            return MiddleRequest::Send(res.to_owned());
        }

        // No cached response found, continue
        MiddleRequest::Continue
    }

    fn end(&self, req: &Request, res: &Response) {
        // Return if its not ment to be
        // ir if response is already in cache
        if !(self.to_cache)(req) || self.cache.read().unwrap().get(&req.path).is_some() {
            return;
        }

        // Add resource to cache
        self.cache
            .write()
            .unwrap()
            .insert(req.path.clone(), (res.to_owned(), current_epoch()));
    }
}

#[inline]
fn current_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards!?")
        .as_secs()
}

pub mod to_cache {
    use crate::{internal::path::Path, Request};

    pub fn path_match(req: &Request, paths: &[&str]) -> bool {
        paths.iter().any(|x| {
            Path::new(req.path.clone())
                .match_path((*x).to_owned())
                .is_some()
        })
    }
}
