//! Windows-specific non-blocking I/O utilities.

use std::{
    io::{self, Error, ErrorKind},
    net::{SocketAddr, TcpListener, TcpStream},
    os::windows::{io::AsRawSocket, raw::SOCKET},
    time::Duration,
};

use sys::{winapi, PollFd};

use super::TcpListenerAcceptTimeout;

mod sys;

impl TcpListenerAcceptTimeout for TcpListener {
    fn accept_timeout(&self, timeout: Duration) -> io::Result<(TcpStream, SocketAddr)> {
        let raw_socket = self.as_raw_socket();

        let mut descriptors = [PollFd::new(raw_socket as SOCKET, sys::EVENT_READ)];
        match poll(&mut descriptors, Some(timeout)) {
            SelectResult::SocketsReady(_) => {
                assert!(descriptors[0].has_revent(sys::EVENT_READ));
                self.accept()
            }
            SelectResult::TimedOut => {
                Err(Error::new(ErrorKind::TimedOut, "The operation timed out."))
            }
            SelectResult::Error(err) => Err(Error::from_raw_os_error(err as i32)),
        }
    }
}

/// The result of a select operation.
pub enum SelectResult {
    /// The number of sockets that are ready.
    SocketsReady(u32),
    /// The timeout was reached before any of the sockets became ready.
    TimedOut,
    /// There was an unexpected error.
    Error(u32),
}

/// Waits for the status of one or more sockets to change, or for a timeout to occur.
pub fn poll(descriptors: &mut [PollFd], timeout: Option<Duration>) -> SelectResult {
    let timeout = timeout.map(|x| x.as_millis() as i32).unwrap_or(-1);
    let result = unsafe {
        winapi::WSAPoll(
            descriptors.as_ptr() as *mut _,
            descriptors.len() as u32,
            timeout,
        )
    };

    match result {
        0 => SelectResult::TimedOut,
        1.. => SelectResult::SocketsReady(result as u32),
        -1 => SelectResult::Error(unsafe { winapi::WSAGetLastError() }),
        _ => unreachable!(),
    }
}
