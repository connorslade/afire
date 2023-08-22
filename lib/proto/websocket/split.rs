use std::{
    fmt::Display,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{Iter, Receiver, SyncSender},
        Arc,
    },
};

use super::{TxType, TxTypeInternal};

/// The sender half of a WebSocket stream.
/// Created by calling [`WebSocketStream::split`] on a [`WebSocketStream`].
#[derive(Clone)]
pub struct WebSocketStreamSender {
    pub(super) tx: Arc<SyncSender<TxTypeInternal>>,
    pub(super) open: Arc<AtomicBool>,
}

/// The receiver half of a WebSocket stream.
/// Created by calling [`WebSocketStream::split`] on a [`WebSocketStream`].
#[derive(Clone)]
pub struct WebSocketStreamReceiver {
    pub(super) rx: Arc<Receiver<TxType>>,
    pub(super) open: Arc<AtomicBool>,
}

impl WebSocketStreamSender {
    /// Sends 'text' data to the client.
    pub fn send(&self, data: impl Display) {
        let _ = self.tx.send(TxType::Text(data.to_string()).into());
    }

    /// Sends binary data to the client.
    pub fn send_binary(&self, data: Vec<u8>) {
        let _ = self.tx.send(TxType::Binary(data).into());
    }

    /// Returns whether the WebSocket stream is open.
    pub fn is_open(&self) -> bool {
        self.open.load(Ordering::Relaxed)
    }
}

impl WebSocketStreamReceiver {
    /// Returns whether the WebSocket stream is open.
    pub fn is_open(&self) -> bool {
        self.open.load(Ordering::Relaxed)
    }
}

impl<'a> IntoIterator for &'a WebSocketStreamReceiver {
    type Item = TxType;
    type IntoIter = Iter<'a, TxType>;

    fn into_iter(self) -> Iter<'a, TxType> {
        self.rx.iter()
    }
}
