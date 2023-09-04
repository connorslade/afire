use std::convert::TryInto;

use super::frame::{Frame, OpCode};

/// Combines multiple frames into a single message.
/// This is used to handle fragmented messages.
pub struct Message {
    pub opcode: OpCode,
    pub payload: Vec<u8>,
}

/// Holds non-final frames until the final frame is received.
/// Then combines the frames into a single [`Message`].
pub struct FrameStack {
    frames: Vec<Frame>,
}

impl FrameStack {
    pub fn new() -> Self {
        Self { frames: Vec::new() }
    }

    /// Adds a frame to the stack.
    /// Returns a [`Message`] if the frame is the final frame in the message.
    pub fn push(&mut self, frame: Frame) -> Option<Message> {
        if !frame.fin {
            self.frames.push(frame);
            return None;
        }

        if self.frames.is_empty() {
            Some(frame.into())
        } else {
            self.frames.push(frame);
            let mut payload = Vec::new();
            for frame in self.frames.drain(..) {
                payload.extend_from_slice(&frame.payload);
            }
            Some(Message {
                opcode: self.frames[0].opcode.try_into().ok()?,
                payload,
            })
        }
    }
}

impl From<Frame> for Message {
    fn from(value: Frame) -> Self {
        Message {
            opcode: value.opcode.try_into().unwrap(),
            payload: value.payload,
        }
    }
}
