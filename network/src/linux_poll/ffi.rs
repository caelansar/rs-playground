use crate::cvt;
use libc::c_int;
use std::io;

pub(crate) fn epoll_create() -> io::Result<i32> {
    // Size argument is ignored but must be greater than zero
    cvt!(libc::epoll_create(1))
}

pub(crate) fn close_fd(fd: i32) -> io::Result<i32> {
    cvt!(libc::close(fd as c_int))
}

pub(crate) fn epoll_ctl(
    epfd: i32,
    op: i32,
    fd: i32,
    event: &mut libc::epoll_event,
) -> io::Result<i32> {
    cvt!(libc::epoll_ctl(
        epfd,
        op,
        fd,
        event as *mut libc::epoll_event
    ))
}

/// Waits for events on the epoll instance to occur. Returns the number file descriptors ready for the requested I/O.
/// When successful, epoll_wait() returns the number of file descriptors ready for the requested
/// I/O, or zero if no file descriptor became ready during the requested timeout milliseconds
pub(crate) fn epoll_wait(
    epfd: i32,
    events: &mut [libc::epoll_event],
    maxevents: i32,
    timeout: i32,
) -> io::Result<i32> {
    cvt!(libc::epoll_wait(
        epfd,
        events.as_mut_ptr(),
        maxevents,
        timeout
    ))
}
