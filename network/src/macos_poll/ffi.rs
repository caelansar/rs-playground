use std::os::fd::RawFd;

use crate::Token;

pub const EVFILT_READ: i16 = -1;
pub const EVFILT_WRITE: i16 = -2;
pub const EVFILT_TIMER: i16 = -7;
pub const EV_ADD: u16 = 0x1;
pub const EV_ENABLE: u16 = 0x4;
pub const EV_ONESHOT: u16 = 0x10;
pub const EV_CLEAR: u16 = 0x20;

#[derive(Debug)]
#[repr(C)]
pub(super) struct Timespec {
    /// Seconds
    tv_sec: isize,
    /// Nanoseconds
    v_nsec: usize,
}

impl Timespec {
    pub fn from_millis(milliseconds: i32) -> Self {
        let seconds = milliseconds / 1000;
        let nanoseconds = (milliseconds % 1000) * 1000 * 1000;
        Timespec {
            tv_sec: seconds as isize,
            v_nsec: nanoseconds as usize,
        }
    }
}

pub type Event = Kevent;
impl Event {
    pub fn new_read_event(fd: RawFd, id: u64) -> Self {
        Event {
            ident: fd as u64,
            filter: EVFILT_READ,
            flags: EV_ADD | EV_ENABLE | EV_ONESHOT,
            fflags: 0,
            data: 0,
            udata: id,
        }
    }

    pub fn new_write_event(fd: RawFd, id: u64) -> Self {
        Event {
            ident: fd as u64,
            filter: EVFILT_WRITE,
            flags: EV_ADD | EV_ONESHOT,
            fflags: 0,
            data: 0,
            udata: id,
        }
    }

    pub fn new_wakeup_event() -> Self {
        Event {
            ident: 0,
            filter: EVFILT_TIMER,
            flags: EV_ADD | EV_ENABLE | EV_CLEAR,
            fflags: 0,
            // data is where our timeout will be set but we want to timeout immideately
            data: 0,
            udata: 0, // TODO: see if windows needs u32...
        }
    }

    pub fn zero() -> Self {
        Event {
            ident: 0,
            filter: 0,
            flags: 0,
            fflags: 0,
            data: 0,
            udata: 0,
        }
    }
}

// https://github.com/rust-lang/libc/blob/c8aa8ec72d631bc35099bcf5d634cf0a0b841be0/src/unix/bsd/apple/mod.rs#L497
// https://github.com/rust-lang/libc/blob/c8aa8ec72d631bc35099bcf5d634cf0a0b841be0/src/unix/bsd/apple/mod.rs#L207
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct Kevent {
    pub ident: u64,
    pub filter: i16,
    pub flags: u16,
    pub fflags: u32,
    pub data: i64,
    pub udata: u64,
}

impl Kevent {
    pub fn token(&self) -> Option<Token> {
        // we have no realiable way of checking if this value is initialized or not but need
        // an option to be compatible with windows.
        Some(self.udata as usize)
    }
}

#[link(name = "c")]
extern "C" {
    /// Returns: positive: file descriptor, negative: error
    pub(super) fn kqueue() -> i32;
    /// Returns: nothing, all non zero return values is an error
    /// If the time limit expires, then kevent() returns 0
    pub(super) fn kevent(
        kq: i32,
        changelist: *const Kevent,
        nchanges: i32,
        eventlist: *mut Kevent,
        nevents: i32,
        timeout: *const Timespec,
    ) -> i32;

    pub fn close(d: i32) -> i32;
}
