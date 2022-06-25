//! Hold methods for interacting with a TCP socket
//! Redefining the functions would allow adding TLS support to a afire server or other low level stuff

use std::io::{self, Read, Write};
use std::net::TcpStream;

/// Hold TCP socket read and write operations
// #[derive(Clone, Copy)]
pub struct SocketHandler {
    /// Function for reading from a tcp socket
    pub socket_read: Box<dyn Fn(&mut TcpStream, &mut Vec<u8>) -> io::Result<usize> + Send + Sync>,

    /// Function for reading an exact ammout of bytes from a TCP socket
    pub socket_read_exact:
        Box<dyn Fn(&mut TcpStream, &mut Vec<u8>) -> io::Result<()> + Send + Sync>,

    /// Function for flushing a TCP socket
    pub socket_flush: Box<dyn Fn(&mut TcpStream) -> io::Result<()> + Send + Sync>,

    /// Function for writing to a TCP socket
    pub socket_write: Box<dyn Fn(&mut TcpStream, &[u8]) -> io::Result<()> + Send + Sync>,
}

impl Default for SocketHandler {
    fn default() -> Self {
        Self {
            socket_read: Box::new(|x, buff| x.read(buff)),
            socket_read_exact: Box::new(|x, buff| x.read_exact(buff)),
            socket_flush: Box::new(|x| x.flush()),
            socket_write: Box::new(|x, y| x.write_all(y)),
        }
    }
}
