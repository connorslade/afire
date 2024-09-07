//! Windows-specific non-blocking I/O utilities.

use std::{
    io::{self, Error, ErrorKind},
    net::{SocketAddr, TcpListener, TcpStream},
    os::windows::{io::AsRawSocket, raw::SOCKET},
    time::Duration,
};

use sys::{winapi, FdSet, TimeVal};

use super::TcpListenerAcceptTimeout;

mod sys;

impl TcpListenerAcceptTimeout for TcpListener {
    fn accept_timeout(&self, timeout: Duration) -> io::Result<(TcpStream, SocketAddr)> {
        let raw_socket = self.as_raw_socket();
        match select(&[raw_socket], &[], &[], timeout) {
            SelectResult::SocketsReady(_) => self.accept(),
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

/// Waits for one of the sockets to become readable, writable, or in an exceptional state, or for the timeout to expire.
pub fn select(
    read: &[SOCKET],
    write: &[SOCKET],
    except: &[SOCKET],
    timeout: Duration,
) -> SelectResult {
    let mut timeout = TimeVal::from_duration(timeout);

    let mut read_fds = FdSet::from_slice(read);
    let mut write_fds = FdSet::from_slice(write);
    let mut except_fds = FdSet::from_slice(except);

    let result = unsafe {
        winapi::select(
            0,
            &mut read_fds,
            &mut write_fds,
            &mut except_fds,
            &mut timeout,
        )
    };

    match result {
        1.. => SelectResult::SocketsReady(result as u32),
        0 => SelectResult::TimedOut,
        ..0 => SelectResult::Error(unsafe { winapi::WSAGetLastError() }),
    }
}
