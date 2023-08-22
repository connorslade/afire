//! WebSocket support.
//! Work in progress.

use std::{
    fmt::Display,
    io::{ErrorKind, Read},
    net::Shutdown,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Iter, Receiver, SyncSender},
        Arc,
    },
    thread,
};

use crate::{
    consts::BUFF_SIZE,
    error::Result,
    internal::{
        encoding::{base64, sha1},
        sync::ForceLockMutex,
    },
    trace::LazyFmt,
    websocket::frame_stack::FrameStack,
    Context, Error, HeaderType, Request, Response, Status,
};

use self::{
    frame::Frame,
    split::{WebSocketStreamReceiver, WebSocketStreamSender},
};

mod frame;
mod frame_stack;
mod split;

const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// A WebSocket stream.
#[derive(Clone)]
pub struct WebSocketStream {
    rx: Arc<Receiver<TxType>>,
    tx: Arc<SyncSender<TxTypeInternal>>,
    open: Arc<AtomicBool>,
}

/// Types of WebSocket frames
#[derive(Debug)]
pub enum TxType {
    /// Close the socket
    Close,
    /// Send / Receive a text message
    Text(String),
    /// Send / Receive a binary message
    Binary(Vec<u8>),
}

#[derive(Debug, PartialEq, Eq)]
enum TxTypeInternal {
    Close,
    Text(String),
    Binary(Vec<u8>),
    Ping,
    Pong,
}

impl WebSocketStream {
    /// Create a new WebSocket stream from a Request.
    pub fn from_request(req: &Request) -> Result<Self> {
        let Some(ws_key) = req.headers.get("Sec-WebSocket-Key").map(|x| x.to_owned()) else {
            return Err(Error::Io("Missing Sec-WebSocket-Key` Header.".to_owned()));
        };
        trace!(Level::Debug, "[WS] Key: {}", ws_key);
        let accept = base64::encode(&sha1::hash((ws_key + WS_GUID).as_bytes()));
        trace!(Level::Debug, "[WS] Accept: {}", accept);

        let mut upgrade = Response::new()
            .status(Status::SwitchingProtocols)
            .header(HeaderType::Upgrade, "websocket")
            .header(HeaderType::Connection, "Upgrade")
            .header("Sec-WebSocket-Accept", &accept)
            .header("Sec-WebSocket-Version", "13");
        // TODO: Get default headers here? somehow?
        upgrade.write(req.socket.clone(), &[])?;

        let open = Arc::new(AtomicBool::new(true));
        let (s2c, rx) = mpsc::sync_channel::<TxTypeInternal>(10);
        let (tx, c2s) = mpsc::sync_channel::<TxType>(10);
        let (s2c, c2s) = (Arc::new(s2c), Arc::new(c2s));
        let this_s2c = s2c.clone();
        let this_open = open.clone();

        let socket = req.socket.force_lock();
        let mut read_socket = socket.try_clone()?;
        let mut write_socket = socket.try_clone()?;
        drop(socket);

        thread::spawn(move || {
            let mut frame_stack = FrameStack::new();
            let mut buffer = Vec::with_capacity(BUFF_SIZE);
            loop {
                let mut buf = vec![0; BUFF_SIZE];
                let len = match read_socket.read(&mut buf) {
                    Ok(l) => l,
                    Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                    Err(e) => {
                        trace!(Level::Debug, "[WS] Read error: {}", e);
                        this_s2c.send(TxTypeInternal::Close).unwrap();
                        break;
                    }
                };

                trace!(Level::Debug, "[WS] Read {} bytes", len);
                if len == 0 {
                    break;
                }

                buffer.extend_from_slice(&buf[..len]);
                if len == BUFF_SIZE {
                    continue;
                }

                trace!(Level::Debug, "[WS] Received: {:?}", &buffer);
                let frame = match Frame::from_slice(&buffer) {
                    Some(f) => f,
                    None => {
                        trace!(Level::Debug, "[WS] Invalid frame");
                        continue;
                    }
                };

                debug_assert_eq!(
                    &buffer,
                    &frame.to_bytes(),
                    "Recoded frame does not match original frame."
                );
                buffer.clear();

                if frame.rsv != 0 {
                    trace!(Level::Trace, "[WS] Received frame with non-zero RSV bits");
                }

                // The frame stack is for handling fragmented messages
                if let Some(frame) = frame_stack.push(frame) {
                    match frame.opcode {
                        // Continuation
                        0 => todo!(),
                        // Text
                        1 => tx
                            .send(TxType::Text(
                                String::from_utf8_lossy(&frame.payload).to_string(),
                            ))
                            .unwrap(),
                        // Binary
                        2 => tx.send(TxType::Binary(frame.payload)).unwrap(),
                        // Close
                        8 => {
                            if !frame.payload.is_empty() {
                                trace!(
                                    Level::Debug,
                                    "[WS] Received close frame with close reason: `{}`",
                                    LazyFmt(|| String::from_utf8_lossy(&frame.payload))
                                );
                            } else {
                                trace!(Level::Debug, "[WS] Received close frame");
                            }
                            this_open.store(false, Ordering::Relaxed);
                            this_s2c.send(TxTypeInternal::Close).unwrap()
                        }
                        // Ping
                        // TODO: Pongs echo the payload of the ping
                        9 => this_s2c.send(TxTypeInternal::Pong).unwrap(),
                        // Pong
                        10 => {}
                        _ => {}
                    }
                }
            }
        });

        thread::spawn(move || {
            for i in rx {
                trace!(Level::Debug, "[WS] Sending {:?}", i);
                let close = i == TxTypeInternal::Close;
                match i {
                    TxTypeInternal::Close => Frame::close(),
                    TxTypeInternal::Text(s) => Frame::text(s),
                    TxTypeInternal::Binary(b) => Frame::binary(b),
                    TxTypeInternal::Ping => Frame::ping(),
                    TxTypeInternal::Pong => Frame::pong(),
                }
                .write(&mut write_socket)
                .unwrap();
                trace!(Level::Debug, "[WS] Sent :p");

                if close {
                    let _ = write_socket.shutdown(Shutdown::Both);
                    break;
                }
            }
        });

        Ok(Self {
            rx: c2s,
            tx: s2c,
            open,
        })
    }

    /// Splits the WebSocket stream into an independent sender and receiver.
    pub fn split(self) -> (WebSocketStreamSender, WebSocketStreamReceiver) {
        (
            WebSocketStreamSender {
                tx: self.tx.clone(),
                open: self.open.clone(),
            },
            WebSocketStreamReceiver {
                rx: self.rx.clone(),
                open: self.open,
            },
        )
    }

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

impl<'a> IntoIterator for &'a WebSocketStream {
    type Item = TxType;
    type IntoIter = Iter<'a, TxType>;

    fn into_iter(self) -> Iter<'a, TxType> {
        self.rx.iter()
    }
}

impl From<TxType> for TxTypeInternal {
    fn from(value: TxType) -> Self {
        match value {
            TxType::Close => TxTypeInternal::Close,
            TxType::Text(s) => TxTypeInternal::Text(s),
            TxType::Binary(b) => TxTypeInternal::Binary(b),
        }
    }
}

/// A trait for initiating a WebSocket connection on a request.
pub trait WebSocketExt {
    /// Initiates a WebSocket connection.
    fn ws(&self) -> Result<WebSocketStream>;
}

impl<T: Send + Sync> WebSocketExt for Context<T> {
    fn ws(&self) -> Result<WebSocketStream> {
        self.req.ws()
    }
}

impl WebSocketExt for Request {
    fn ws(&self) -> Result<WebSocketStream> {
        self.socket.set_raw(true);
        WebSocketStream::from_request(self)
    }
}

fn xor_mask(mask: &[u8], data: &[u8]) -> Vec<u8> {
    debug_assert_eq!(mask.len(), 4);

    let mut decoded = Vec::with_capacity(data.len());
    for (i, byte) in data.iter().enumerate() {
        decoded.push(byte ^ mask[i % 4]);
    }

    decoded
}
