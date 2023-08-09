use std::{
    io::{self, Read},
    net::TcpStream,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, RwLock},
};

use crate::internal::sync::{ForceLockMutex, ForceLockRwLock, SingleBarrier};

/// Socket is a wrapper around TcpStream that allows for sending a response from another thread.
pub struct Socket {
    /// The internal TcpStream.
    pub socket: Mutex<TcpStream>,
    /// A barrier that is used to wait for the response to be sent in the case of a guaranteed send.
    /// This allows for sending a response from another thread, not sure why you would want to do that though.
    pub(crate) barrier: RwLock<Arc<Option<SingleBarrier>>>,
}

impl Socket {
    pub(crate) fn new(socket: TcpStream) -> Self {
        Self {
            socket: Mutex::new(socket),
            barrier: RwLock::new(Arc::new(None)),
        }
    }

    pub(crate) fn unlock(&self) {
        let barrier = self.barrier.force_read().clone();
        if let Some(i) = &*barrier {
            i.unlock();
        }
    }

    pub(crate) fn add_barrier(&self) {
        *self.barrier.force_write() = Arc::new(Some(SingleBarrier::new()));
    }
}

impl Deref for Socket {
    type Target = Mutex<TcpStream>;

    fn deref(&self) -> &Self::Target {
        &self.socket
    }
}
