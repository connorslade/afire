use std::sync::{atomic::Ordering, Arc};

use super::config::ServerConfig;

#[derive(Clone)]
pub struct ServerHandle<State: 'static + Send + Sync> {
    pub(super) config: Arc<ServerConfig>,
    pub(super) state: Arc<State>,
}

impl<State: Send + Sync + 'static> ServerHandle<State> {
    pub fn app(&self) -> &Arc<State> {
        &self.state
    }

    pub fn shutdown(&self) {
        self.config.running.store(false, Ordering::Relaxed);
    }
}
