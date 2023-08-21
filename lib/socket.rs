use std::{
    net::TcpStream,
    ops::Deref,
    sync::{Arc, Mutex, RwLock},
};

use crate::{
    internal::sync::{ForceLockRwLock, SingleBarrier},
    response::ResponseFlag,
};

/// Socket is a wrapper around TcpStream that allows for sending a response from another thread.
pub struct Socket {
    /// The internal TcpStream.
    pub socket: Mutex<TcpStream>,
    /// A barrier that is used to wait for the response to be sent in the case of a guaranteed send.
    /// This allows for sending a response from another thread, not sure why you would want to do that though.
    pub(crate) barrier: Arc<SingleBarrier>,
    pub(crate) flag: RwLock<ResponseFlag>,
}

impl Socket {
    pub(crate) fn new(socket: TcpStream) -> Self {
        Self {
            socket: Mutex::new(socket),
            barrier: Arc::new(SingleBarrier::new()),
            flag: RwLock::new(ResponseFlag::None),
        }
    }

    pub(crate) fn unlock(&self) {
        self.barrier.unlock();
    }

    pub(crate) fn reset_barrier(&self) {
        self.barrier.reset();
    }

    pub(crate) fn flag(&self) -> ResponseFlag {
        *self.flag.force_read()
    }

    pub(crate) fn set_flag(&self, flag: ResponseFlag) {
        *self.flag.force_write() = flag;
    }
}

impl Deref for Socket {
    type Target = Mutex<TcpStream>;

    fn deref(&self) -> &Self::Target {
        &self.socket
    }
}
