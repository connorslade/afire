use std::os::windows::raw::SOCKET;

#[repr(C)]
pub struct PollFd {
    socket: SOCKET,
    events: i16,
    revents: i16,
}

pub mod consts {
    #![allow(unused)]

    pub const EVENT_PRIORITY_READ: i16 = 0x0400;
    pub const EVENT_OOB_READ: i16 = 0x0200;
    pub const EVENT_READ: i16 = 0x0100;
    pub const EVENT_WRITE: i16 = 0x0010;
    pub const EVENT_ERROR: i16 = 0x0001;
    pub const EVENT_DISCONNECT: i16 = 0x0002;
    pub const EVENT_INVALID: i16 = 0x0004;
}

pub mod winapi {
    use super::*;

    extern "system" {
        /// The WSAPoll function determines status of one or more sockets.
        /// See [Win32 Docs](https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-wsapoll).
        pub fn WSAPoll(fd_array: *mut PollFd, fds: u32, timeout: i32) -> i32;

        /// The WSAGetLastError function returns the error status for the last Windows Sockets operation that failed.
        /// See [Win32 Docs](https://learn.microsoft.com/en-us/windows/win32/api/winsock/nf-winsock-wsagetlasterror).
        pub fn WSAGetLastError() -> u32;
    }
}

impl PollFd {
    pub fn new(socket: SOCKET, events: i16) -> PollFd {
        PollFd {
            socket,
            events,
            revents: 0,
        }
    }

    pub fn has_revent(&self, events: i16) -> bool {
        self.revents & events != 0
    }
}
