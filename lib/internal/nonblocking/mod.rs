//! Utilities for non-blocking I/O.

use std::{
    io,
    net::{SocketAddr, TcpStream},
    time::Duration,
};

#[cfg(windows)]
pub mod windows;

/// Extension trait for [`TcpListener`] to add a timeout to the [`accept`] method.
pub trait TcpListenerAcceptTimeout {
    /// Accept a connection with a timeout.
    fn accept_timeout(&self, timeout: Duration) -> io::Result<(TcpStream, SocketAddr)>;
}
