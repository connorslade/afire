use std::{
    net::TcpStream,
    ops::Deref,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex, RwLock,
    },
};

use crate::{
    internal::sync::{ForceLockRwLock, SingleBarrier},
    response::ResponseFlag,
};

/// Socket is a wrapper around TcpStream that allows for sending a response from other threads.
pub struct Socket {
    /// The internal TcpStream.
    pub socket: Mutex<TcpStream>,
    /// A unique identifier that uniquely identifies this socket.
    pub id: u64,
    /// A barrier that is used to wait for the response to be sent in the case of a guaranteed send.
    /// This allows for sending a response from another thread, not sure why you would want to do that though.
    pub(crate) barrier: Arc<SingleBarrier>,
    /// If true, the socket is being handled by another system.
    /// This could be SSE or WebSockets, but either way afire core should not mess with it.
    pub(crate) raw: AtomicBool,
    // TODO: work on this
    /// If the socket should be closed after the response is sent.
    pub(crate) flag: RwLock<ResponseFlag>,
}

impl Socket {
    /// Create a new `Socket` from a `TcpStream`.
    /// Will also create a new unique identifier for the socket.
    pub(crate) fn new(socket: TcpStream) -> Self {
        static ID: AtomicU64 = AtomicU64::new(0);
        Self {
            socket: Mutex::new(socket),
            id: ID.fetch_add(1, Ordering::Relaxed),
            barrier: Arc::new(SingleBarrier::new()),
            raw: AtomicBool::new(false),
            flag: RwLock::new(ResponseFlag::None),
        }
    }

    /// Unlock the barrier.
    /// This indicates that the response has been sent, and the thread that was waiting for the response can continue.
    pub(crate) fn unlock(&self) {
        self.barrier.unlock();
    }

    /// Re-lock the barrier.
    pub(crate) fn reset_barrier(&self) {
        self.barrier.reset();
    }

    /// Get the response flag.
    pub(crate) fn flag(&self) -> ResponseFlag {
        *self.flag.force_read()
    }

    /// Set the response flag.
    pub(crate) fn set_flag(&self, flag: ResponseFlag) {
        *self.flag.force_write() = flag;
    }

    /// Check if the socket is being handled by another system.
    pub fn is_raw(&self) -> bool {
        self.raw.load(Ordering::Relaxed)
    }

    /// Set the socket as being handled by another system.
    pub(crate) fn set_raw(&self, raw: bool) {
        self.raw.store(raw, Ordering::Relaxed);
    }
}

impl Deref for Socket {
    type Target = Mutex<TcpStream>;

    fn deref(&self) -> &Self::Target {
        &self.socket
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        trace!(Level::Debug, "Dropping socket {}", self.id);
    }
}
