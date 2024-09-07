use std::{os::windows::raw::SOCKET, time::Duration};

#[repr(C)]
pub struct FdSet {
    fd_count: u32,
    fd_array: [SOCKET; 64],
}

#[repr(C)]
pub struct TimeVal {
    tv_sec: i64,
    tv_usec: i64,
}

pub mod winapi {
    use super::*;

    extern "system" {
        /// The select function determines the status of one or more sockets, waiting if necessary, to perform synchronous I/O.
        /// See [Win32 Docs](https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-select).
        pub fn select(
            n: i32,
            read: *mut FdSet,
            write: *mut FdSet,
            except: *mut FdSet,
            timeout: *mut TimeVal,
        ) -> i32;

        /// The WSAGetLastError function returns the error status for the last Windows Sockets operation that failed.
        /// See [Win32 Docs](https://learn.microsoft.com/en-us/windows/win32/api/winsock/nf-winsock-wsagetlasterror).
        pub fn WSAGetLastError() -> u32;
    }
}

impl FdSet {
    /// If more than 64 sockets are passed, only the first 64 will be used.
    pub fn from_slice(slice: &[SOCKET]) -> Self {
        let mut fd_array = [0; 64];
        for (i, &socket) in slice.iter().take(64).enumerate() {
            fd_array[i] = socket;
        }

        Self {
            fd_count: slice.len() as u32,
            fd_array,
        }
    }
}

impl TimeVal {
    pub fn from_duration(duration: Duration) -> Self {
        let secs = duration.as_secs();
        let usecs = duration.subsec_micros();
        Self {
            tv_sec: secs as i64,
            tv_usec: usecs as i64,
        }
    }
}
