//! Hold methods for interacting with a TCP socket
//! Redefining the functions would allow adding TLS support to a afire server or other low level stuff

use std::io::{Read, Result, Write};
use std::net::TcpStream;

/// Hold TCP socket read and write operations
#[derive(Clone, Copy)]
pub struct SocketHandler {
    /// Function for reading from a tcp socket
    pub socket_read: fn(&mut TcpStream, &mut Vec<u8>) -> Result<usize>,

    /// Function for reading an exact ammout of bytes from a TCP socket
    pub socket_read_exact: fn(&mut TcpStream, &mut Vec<u8>) -> Result<()>,

    /// Function for flushing a TCP socket
    pub socket_flush: fn(&mut TcpStream) -> Result<()>,

    /// Function for writing to a TCP socket
    pub socket_write: fn(&mut TcpStream, &[u8]) -> Result<()>,
}

impl Default for SocketHandler {
    fn default() -> Self {
        Self {
            socket_read: |x, buff| x.read(buff),
            socket_read_exact: |x, buff| x.read_exact(buff),
            socket_flush: |x| x.flush(),
            socket_write: |x, y| x.write_all(y),
        }
    }
}
