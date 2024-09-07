//! The event loop used to handle incoming connections.

use std::{
    net::{SocketAddr, TcpListener},
    sync::{atomic::Ordering, Arc},
    time::Duration,
};

use crate::{
    error::Result,
    internal::{handle::handle, socket::Socket},
    trace, Server,
};

use super::nonblocking::TcpListenerAcceptTimeout;

/// afire servers are event-driven.
/// This trait defines the event loop that will be used to handle incoming connections.
/// The default implementation is [`TcpEventLoop`], which as the name suggests, uses the standard library built-in TCP listener.
///
/// This trait exists to allow for custom event loops to be used, with the use of the [`Socket`] trait, one could for example:
/// - Use a different protocol, such as UDP
/// - Use a different wrapped socket type, such as a TLS socket.
///   This is the main reason for this trait, my crate [afire-tls](https://crates.io/crates/afire-tls) uses this.
pub trait EventLoop<State: Send + Sync> {
    /// Run the event loop.
    /// The event loop should accept connections from `addr` and handle them using `server`.
    fn run(&self, server: Arc<Server<State>>, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr)?;
        loop {
            if !server.running.load(Ordering::Relaxed) {
                trace!(
                    Level::Debug,
                    "Stopping event loop. No more connections will be accepted."
                );
                break;
            }

            #[cfg(windows)]
            let result = listener.accept_timeout(Duration::from_secs(1));
            #[cfg(not(windows))]
            let result = listener.accept();

            match result {
                Ok(event) => {
                    let this_server = server.clone();
                    let event = Arc::new(Socket::new(event.0));
                    server.thread_pool.execute(|| handle(event, this_server));
                }
                Err(err) => trace!(Level::Error, "Error accepting connection: {err}"),
            };
        }
        Ok(())
    }
}

/// The default event loop.
/// Uses the standard library built-in TCP listener.
pub struct TcpEventLoop;

impl<State: Send + Sync> EventLoop<State> for TcpEventLoop {}
