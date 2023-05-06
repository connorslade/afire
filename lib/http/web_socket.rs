use std::{
    convert::TryInto,
    io::{self, Read, Write},
    net::TcpStream,
    sync::{
        mpsc::{self, Receiver, Sender, SyncSender},
        Arc, Mutex,
    },
    thread,
};

use crate::{
    internal::{
        common::ForceLock,
        encoding::{base64, sha1},
    },
    HeaderType, Request, Response, Status,
};

const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub struct WebSocketStream {
    _rx: Arc<Receiver<TxType>>,
    _tx: Arc<SyncSender<TxType>>,
}

#[derive(Debug)]
struct Frame {
    fin: bool,
    /// RSV1, RSV2, RSV3
    /// BitPacked into one byte (0xRRR)
    rsv: u8,
    opcode: u8,
    payload_len: u64,
    mask: Option<[u8; 4]>,
    payload: Vec<u8>,
}

#[derive(Debug)]
enum TxType {
    Close,
}

#[derive(Debug)]
enum RxType {
    Text(String),
    Binary(Vec<u8>),
}

impl WebSocketStream {
    /// Create a new WebSocket stream from a Request.
    pub fn from_request(req: &Request) -> io::Result<Self> {
        dbg!(&req);
        let ws_key = req.headers.get("Sec-WebSocket-Key").unwrap().to_owned();
        trace!(Level::Debug, "WS Key: {}", ws_key);
        let accept = base64::encode(&sha1::hash((ws_key + WS_GUID).as_bytes()));
        trace!(Level::Debug, "WS Accept: {}", accept);

        let mut upgrade = Response::new()
            .status(Status::SwitchingProtocols)
            .header(HeaderType::Upgrade, "websocket")
            .header(HeaderType::Connection, "Upgrade")
            .header("Sec-WebSocket-Accept", &accept)
            .header("Sec-WebSocket-Version", "13");
        upgrade.write(req.socket.clone(), &[]).unwrap();

        let (s2c, rx) = mpsc::sync_channel::<TxType>(10);
        let (_tx, c2s) = mpsc::sync_channel::<TxType>(10);
        let (s2c, c2s) = (Arc::new(s2c), Arc::new(c2s));

        let socket = req.socket.clone();
        let this_s2c = s2c.clone();
        thread::spawn(move || {
            let mut socket = socket.force_lock();
            let mut buf = [0u8; 1024];
            loop {
                let len = socket.read(&mut buf).unwrap();
                if len == 0 {
                    break;
                }

                let frame = match Frame::from_slice(&buf[..len]) {
                    Some(f) => f,
                    None => continue,
                };

                if !frame.fin {
                    todo!("Handle fragmented frames");
                }

                if frame.rsv != 0 {
                    trace!(Level::Trace, "WS: Received frame with non-zero RSV bits");
                }

                // 0 = continuation
                // 1 = text
                // 2 = binary
                // 8 = close
                // 9 = ping
                // 10 = pong
                match frame.opcode {
                    0 => {}
                    1 => {}
                    2 => {}
                    8 => {
                        this_s2c.send(TxType::Close).unwrap();
                    }
                    9 => {}
                    10 => {}
                    _ => {}
                }
            }
        });

        let socket = req.socket.clone();
        thread::spawn(move || {
            //todo
            for i in rx {
                trace!(Level::Debug, "WS: Sending {:?}", i);
                match i {
                    TxType::Close => {
                        Frame::close().write(socket.clone()).unwrap();
                    }
                }
                trace!(Level::Debug, "WS: Sent :p");
            }
        });

        // todo: everything else :sweat_smile:\

        Ok(Self { _rx: c2s, _tx: s2c })
    }
}

impl Frame {
    fn from_slice(buf: &[u8]) -> Option<Self> {
        let fin = buf[0] & 0b1000_0000 != 0;
        let rsv = buf[0] & 0b0111_0000 >> 4;

        let mask = buf[1] & 0b1000_0000 != 0;
        let opcode = buf[0] & 0b0000_1111;
        let (payload_len, offset) = match buf[1] as u64 & 0b0111_1111 {
            126 => (u16::from_be_bytes([buf[2], buf[3]]) as u64, 4),
            127 => (
                u64::from_be_bytes([
                    buf[2], buf[3], buf[4], buf[5], buf[6], buf[7], buf[8], buf[9],
                ]),
                10,
            ),
            i => (i, 2),
        };
        trace!(
            Level::Debug,
            "WS: {{ fin: {fin}, rsv: {rsv}, opcode: {opcode}, payload_len: {payload_len}, mask: {mask} }}",
        );

        if payload_len == 0 {
            trace!(Level::Debug, "WS: Empty payload");
            return None;
        }

        if !mask {
            trace!(Level::Debug, "WS: No mask");
            return None;
        }

        let mut decoded = Vec::with_capacity(payload_len as usize);
        let mask = &buf[offset..offset + 4];
        for i in 0..payload_len as usize {
            decoded.push(buf[i + offset + 4] ^ mask[i % 4]);
        }

        trace!(Level::Debug, "WS: Decoded: {:?}", decoded);
        trace!(
            Level::Debug,
            "WS: Decoded: {:?}",
            String::from_utf8_lossy(&decoded)
        );

        Some(Self {
            fin,
            rsv,
            opcode,
            payload_len,
            mask: Some(mask.try_into().unwrap()),
            payload: decoded,
        })
    }

    /*
      0                   1                   2                   3
      0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
     +-+-+-+-+-------+-+-------------+-------------------------------+
     |F|R|R|R| opcode|M| Payload len |    Extended payload length    |
     |I|S|S|S|  (4)  |A|     (7)     |             (16/64)           |
     |N|V|V|V|       |S|             |   (if payload len==126/127)   |
     | |1|2|3|       |K|             |                               |
     +-+-+-+-+-------+-+-------------+ - - - - - - - - - - - - - - - +
     |     Extended payload length continued, if payload len == 127  |
     + - - - - - - - - - - - - - - - +-------------------------------+
     |                               |Masking-key, if MASK set to 1  |
     +-------------------------------+-------------------------------+
     | Masking-key (continued)       |          Payload Data         |
     +-------------------------------- - - - - - - - - - - - - - - - +
     :                     Payload Data continued ...                :
     + - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - +
     |                     Payload Data continued ...                |
     +---------------------------------------------------------------+
    */
    fn write(&self, socket: Arc<Mutex<TcpStream>>) -> io::Result<()> {
        let mut buf = Vec::new();
        buf.push((self.fin as u8) << 7 | self.rsv << 4 | self.opcode);

        if self.payload_len < 126 {
            buf.push((self.mask.is_some() as u8) << 7 | self.payload_len as u8);
        } else if self.payload_len < 65536 {
            buf.push((self.mask.is_some() as u8) << 7 | 126);
            buf.extend_from_slice(&self.payload_len.to_be_bytes());
        } else {
            buf.push((self.mask.is_some() as u8) << 7 | 127);
            buf.extend_from_slice(&self.payload_len.to_be_bytes());
        }

        if let Some(mask) = self.mask {
            buf.extend_from_slice(&mask);
        }

        buf.extend_from_slice(&self.payload);

        trace!(Level::Debug, "WS: Writing: {:?}", buf);

        socket.force_lock().write(&buf)?;
        Ok(())
    }

    fn close() -> Self {
        Self {
            fin: true,
            rsv: 0,
            opcode: 8,
            payload_len: 0,
            mask: None,
            payload: Vec::new(),
        }
    }

    fn rsv1(&self) -> bool {
        self.rsv & 0b100 != 0
    }

    fn rsv2(&self) -> bool {
        self.rsv & 0b010 != 0
    }

    fn rsv3(&self) -> bool {
        self.rsv & 0b001 != 0
    }
}

/// A trait for initiating a WebSocket connection on a request.
pub trait WebSocketExt {
    /// Initiates a WebSocket connection on a request.
    fn ws(&self) -> io::Result<WebSocketStream>;
}

impl WebSocketExt for Request {
    fn ws(&self) -> io::Result<WebSocketStream> {
        WebSocketStream::from_request(self)
    }
}

fn decode(mask: &[u8], data: &[u8]) -> Vec<u8> {
    debug_assert_eq!(mask.len(), 4);

    let mut decoded = Vec::with_capacity(data.len());
    for (i, byte) in data.iter().enumerate() {
        decoded.push(byte ^ mask[i % 4]);
    }
    decoded
}
