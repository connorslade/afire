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
    context::ContextFlag,
    error::Result,
    internal::{
        encoding::{base64, sha1},
        sync::ForceLockMutex,
    },
    trace::LazyFmt,
    websocket::{frame::OpCode, frame_stack::FrameStack},
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
const CHANNEL_LENGTH: usize = 10;

/// A WebSocket stream.
pub struct WebSocketStream {
    rx: Receiver<TxType>,
    tx: SyncSender<TxTypeInternal>,
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
#[allow(dead_code)]
enum TxTypeInternal {
    Close,
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
}

impl WebSocketStream {
    /// Create a new WebSocket stream from a Request.
    pub fn from_request(req: &Request) -> Result<Self> {
        let Some(ws_key) = req.headers.get("Sec-WebSocket-Key").map(|x| x.to_owned()) else {
            return Error::bail("Missing `Sec-WebSocket-Key` Header.");
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
        trace!(Level::Debug, "[WS] Upgraded socket #{}", req.socket.id);

        let open = Arc::new(AtomicBool::new(true));
        let (s2c, rx) = mpsc::sync_channel::<TxTypeInternal>(CHANNEL_LENGTH);
        let (tx, c2s) = mpsc::sync_channel::<TxType>(CHANNEL_LENGTH);
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
                buffer.extend_from_slice(&buf[..len]);

                // If read returns 0, the socket has been closed
                if len == 0 {
                    this_open.store(false, Ordering::Relaxed);
                    break;
                }

                // Get the length of the payload and the offset of the payload
                let Some((payload_len, offset)) = frame::payload_length(&buffer) else {
                    continue;
                };

                // Continue reading until we have the entire payload
                if buffer.len() < offset + 4 + payload_len as usize {
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
                        // Handled by the frame stack
                        OpCode::Continuation => unreachable!(),
                        OpCode::Text => tx
                            .send(TxType::Text(
                                String::from_utf8_lossy(&frame.payload).to_string(),
                            ))
                            .unwrap(),
                        OpCode::Binary => tx.send(TxType::Binary(frame.payload)).unwrap(),
                        OpCode::Close => {
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
                        OpCode::Ping => this_s2c.send(TxTypeInternal::Pong(frame.payload)).unwrap(),
                        OpCode::Pong => {}
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
                    TxTypeInternal::Ping(b) => Frame::ping(b),
                    TxTypeInternal::Pong(b) => Frame::pong(b),
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
                tx: self.tx,
                open: self.open.clone(),
            },
            WebSocketStreamReceiver {
                rx: self.rx,
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
        if self.flags.get(ContextFlag::ResponseSent) {
            Error::bail("Response already sent.")?;
        }

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

    let mut decoded = data.to_vec();
    for (i, byte) in decoded.iter_mut().enumerate() {
        *byte ^= mask[i % 4]
    }

    decoded
}
