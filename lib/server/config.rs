use std::{sync::atomic::AtomicBool, time::Duration};

use crate::{header::Headers, Header, HeaderName, VERSION};

pub struct ServerConfig {
    /// Headers automatically added to every response.
    pub default_headers: Headers,

    /// Weather to allow keep-alive connections.
    /// If this is set to false, the server will close the connection after every request.
    /// This is enabled by default.
    pub keep_alive: bool,

    /// Socket Timeout
    pub socket_timeout: Option<Duration>,

    /// Weather the server is running.
    /// If this is set to false, the server will stop accepting connections.
    pub running: AtomicBool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            default_headers: Headers(vec![Header::new(
                HeaderName::Server,
                format!("afire/{VERSION}"),
            )]),
            keep_alive: false,
            socket_timeout: None,
            running: AtomicBool::new(true),
        }
    }
}
