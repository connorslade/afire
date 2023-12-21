use std::{
    convert::{TryFrom, TryInto},
    io::{self, Write},
};

use super::xor_mask;
use crate::{internal::socket::SocketStream, trace::LazyFmt};

/// ## Frame Layout
/// ```plain
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-------+-+-------------+-------------------------------+
/// |F|R|R|R| opcode|M| Payload len |    Extended payload length    |
/// |I|S|S|S|  (4)  |A|     (7)     |             (16/64)           |
/// |N|V|V|V|       |S|             |   (if payload len==126/127)   |
/// | |1|2|3|       |K|             |                               |
/// +-+-+-+-+-------+-+-------------+ - - - - - - - - - - - - - - - +
/// |     Extended payload length continued, if payload len == 127  |
/// + - - - - - - - - - - - - - - - +-------------------------------+
/// |                               |Masking-key, if MASK set to 1  |
/// +-------------------------------+-------------------------------+
/// | Masking-key (continued)       |          Payload Data         |
/// +-------------------------------- - - - - - - - - - - - - - - - +
/// :                     Payload Data continued ...                :
/// + - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - +
/// |                     Payload Data continued ...                |
/// +---------------------------------------------------------------+
/// ```
#[derive(Debug)]
pub struct Frame {
    pub fin: bool,
    /// RSV1, RSV2, RSV3
    /// BitPacked into one byte (0x123)
    pub rsv: u8,
    pub opcode: u8,
    pub payload_len: u64,
    pub mask: Option<[u8; 4]>,
    pub payload: Vec<u8>,
}

#[repr(u8)]
pub enum OpCode {
    Continuation = 0,
    Text = 1,
    Binary = 2,
    Close = 8,
    Ping = 9,
    Pong = 10,
}

impl Frame {
    pub fn from_slice(buf: &[u8]) -> Option<Self> {
        let fin = buf[0] & 0b1000_0000 != 0;
        let rsv = (buf[0] & 0b0111_0000) >> 4;

        let mask = buf[1] & 0b1000_0000 != 0;
        let opcode = buf[0] & 0b0000_1111;
        let (payload_len, offset) = payload_length(buf)?;

        trace!(
            Level::Debug,
            "[WS] {{ fin: {fin}, rsv: {rsv}, opcode: {opcode}, payload_len: {payload_len}, mask: \
             {mask} }}",
        );

        if payload_len == 0 {
            trace!(Level::Debug, "[WS] Empty payload");
        }

        if !mask {
            trace!(Level::Debug, "[WS] No mask");
            // Are unmasked payloads not allowed?
            // girl i wrote this all so long ago i don't remember
            return None;
        }

        let mut decoded = Vec::with_capacity(payload_len as usize);
        let mask = &buf[offset..offset + 4];
        for i in 0..payload_len as usize {
            decoded.push(buf[i + offset + 4] ^ mask[i % 4]);
        }

        trace!(Level::Debug, "[WS] Decoded: {:?}", decoded);
        trace!(
            Level::Debug,
            "[WS] Decoded: {:?}",
            LazyFmt(|| String::from_utf8_lossy(&decoded))
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

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push((self.fin as u8) << 7 | self.rsv << 4 | self.opcode);

        match self.payload_len {
            ..=125 => buf.push((self.mask.is_some() as u8) << 7 | self.payload_len as u8),
            126..=65535 => {
                buf.push((self.mask.is_some() as u8) << 7 | 126);
                buf.extend_from_slice(&(self.payload_len as u16).to_be_bytes());
            }
            _ => {
                buf.push((self.mask.is_some() as u8) << 7 | 127);
                buf.extend_from_slice(&self.payload_len.to_be_bytes());
            }
        }

        match self.mask {
            Some(mask) => {
                buf.extend_from_slice(&mask);
                buf.extend_from_slice(&xor_mask(&mask, &self.payload))
            }
            None => buf.extend_from_slice(&self.payload),
        }

        buf
    }

    pub fn write(&self, socket: &mut SocketStream) -> io::Result<()> {
        let buf = self.to_bytes();
        trace!(Level::Debug, "[WS] Writing: {:?}", buf);

        socket.write_all(&buf)?;
        Ok(())
    }

    pub fn close() -> Self {
        Self {
            fin: true,
            rsv: 0,
            opcode: 8,
            payload_len: 0,
            mask: None,
            payload: Vec::new(),
        }
    }

    pub fn text(text: String) -> Self {
        Self {
            fin: true,
            rsv: 0,
            opcode: 1,
            payload_len: text.len() as u64,
            mask: None,
            payload: text.into_bytes(),
        }
    }

    pub fn binary(binary: Vec<u8>) -> Self {
        Self {
            fin: true,
            rsv: 0,
            opcode: 2,
            payload_len: binary.len() as u64,
            mask: None,
            payload: binary,
        }
    }

    pub fn ping(binary: Vec<u8>) -> Self {
        Self {
            fin: true,
            rsv: 0,
            opcode: 9,
            payload_len: 0,
            mask: None,
            payload: binary,
        }
    }

    pub fn pong(binary: Vec<u8>) -> Self {
        Self {
            fin: true,
            rsv: 0,
            opcode: 10,
            payload_len: 0,
            mask: None,
            payload: binary,
        }
    }

    pub fn _rsv1(&self) -> bool {
        self.rsv & 0b100 != 0
    }

    pub fn _rsv2(&self) -> bool {
        self.rsv & 0b010 != 0
    }

    pub fn _rsv3(&self) -> bool {
        self.rsv & 0b001 != 0
    }
}

impl From<OpCode> for u8 {
    fn from(opcode: OpCode) -> Self {
        opcode as u8
    }
}

impl TryFrom<u8> for OpCode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => OpCode::Continuation,
            1 => OpCode::Text,
            2 => OpCode::Binary,
            8 => OpCode::Close,
            9 => OpCode::Ping,
            10 => OpCode::Pong,
            _ => return Err(()),
        })
    }
}

/// Returns (payload_len, offset).
/// If not enough bytes are available, returns None.
pub fn payload_length(buf: &[u8]) -> Option<(u64, usize)> {
    if buf.len() < 2 {
        return None;
    }

    let (payload_len, offset) = match buf[1] as u64 & 0b0111_1111 {
        126 => {
            if buf.len() < 4 {
                return None;
            }

            (u16::from_be_bytes([buf[2], buf[3]]) as u64, 4)
        }
        127 => {
            if buf.len() < 10 {
                return None;
            }

            (
                u64::from_be_bytes([
                    buf[2], buf[3], buf[4], buf[5], buf[6], buf[7], buf[8], buf[9],
                ]),
                10,
            )
        }
        i => (i, 2),
    };

    Some((payload_len, offset))
}
