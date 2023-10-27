use std::{io, os::fd::RawFd};

use libc::{c_int, timespec};

use crate::cvt;

pub fn close(fd: RawFd) -> io::Result<i32> {
    cvt!(libc::close(fd))
}

pub fn kqueue() -> io::Result<i32> {
    cvt!(libc::kqueue())
}

pub fn kevent(
    kq: c_int,
    changelist: *const libc::kevent,
    nchanges: c_int,
    eventlist: *mut libc::kevent,
    nevents: c_int,
    timeout: *const timespec,
) -> io::Result<i32> {
    cvt!(libc::kevent(
        kq, changelist, nchanges, eventlist, nevents, timeout
    ))
}
