use std::{net::SocketAddr, sync::atomic::AtomicBool, time::Duration};

use crate::header::Headers;

pub struct ServerConfig {
    /// Socket address to bind to.
    pub host: SocketAddr,

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
