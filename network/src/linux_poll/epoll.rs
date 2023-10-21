use super::ffi;
use crate::{EventID, Interests, TcpStream, Token};
use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::{io, time};

pub type Events = Vec<Event>;

pub struct Registrator {
    fd: RawFd,
    is_poll_dead: Arc<AtomicBool>,
}

impl Registrator {
    pub fn register(
        &self,
        stream: &TcpStream,
        token: usize,
        interests: Interests,
    ) -> io::Result<()> {
        if self.is_poll_dead.load(Ordering::SeqCst) {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "Poll instance closed.",
            ));
        }
        let fd = stream.as_raw_fd();
        if interests.is_readable() {
            // We register the id (or most oftenly referred to as a Token) to the `udata` field
            // if the `Kevent`
            let mut event = ffi::Event::new(ffi::EPOLLIN | ffi::EPOLLONESHOT, token);
            epoll_ctl(self.fd, ffi::EPOLL_CTL_ADD, fd, &mut event)?;
        };

        if interests.is_writable() {
            unimplemented!();
        }

        Ok(())
    }

    pub fn deregister(&self, stream: &TcpStream) -> io::Result<()> {
        println!("unimplemented!");
        Ok(())
    }

    pub fn close_loop(&self) -> io::Result<()> {
        if self
            .is_poll_dead
            .compare_and_swap(false, true, Ordering::SeqCst)
        {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "Poll instance closed.",
            ));
        }

        // This is a little hacky but works for our needs right now
        let wake_fd = eventfd(1, 0)?;
        let mut event = ffi::Event::new(ffi::EPOLLIN, 0);
        epoll_ctl(self.fd, ffi::EPOLL_CTL_ADD, wake_fd, &mut event)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Selector {
    fd: RawFd,
}

impl Selector {
    pub fn new() -> io::Result<Self> {
        Ok(Selector {
            fd: epoll_create()?,
        })
    }

    /// This function blocks and waits until an event has been recieved. `timeout` None means
    /// the poll will never time out.
    pub fn select(
        &self,
        events: &mut Events,
        timeout_ms: Option<time::Duration>,
    ) -> io::Result<()> {
        events.clear();
        let timeout = timeout_ms.unwrap_or(time::Duration::from_millis(100));
        epoll_wait(self.fd, events, 1024, -1).map(|n_events| {
            // This is safe because `syscall_kevent` ensures that `n_events` are
            // assigned. We could check for a valid token for each event to verify so this is
            // just a performance optimization used in `mio` and copied here.
            unsafe { events.set_len(n_events as usize) };
        })
    }

    pub fn registrator(&self, is_poll_dead: Arc<AtomicBool>) -> Registrator {
        Registrator {
            fd: self.fd,
            is_poll_dead,
        }
    }
}

impl Drop for Selector {
    fn drop(&mut self) {
        match close_fd(self.fd) {
            Ok(..) => (),
            Err(e) => {
                if !std::thread::panicking() {
                    panic!("{}", e);
                }
            }
        }
    }
}

pub type Event = ffi::Event;
impl EventID for Event {
    fn id(&self) -> Token {
        self.data()
    }
}

fn epoll_create() -> io::Result<i32> {
    // Size argument is ignored but must be greater than zero
    let res = unsafe { ffi::epoll_create(1) };
    if res < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(res)
    }
}

fn close_fd(fd: i32) -> io::Result<()> {
    let res = unsafe { ffi::close(fd) };
    if res < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn epoll_ctl(epfd: i32, op: i32, fd: i32, event: &mut Event) -> io::Result<()> {
    let res = unsafe { ffi::epoll_ctl(epfd, op, fd, event) };
    if res < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

/// Waits for events on the epoll instance to occur. Returns the number file descriptors ready for the requested I/O.
/// When successful, epoll_wait() returns the number of file descriptors ready for the requested
/// I/O, or zero if no file descriptor became ready during the requested timeout milliseconds
fn epoll_wait(epfd: i32, events: &mut [Event], maxevents: i32, timeout: i32) -> io::Result<i32> {
    let res = unsafe { ffi::epoll_wait(epfd, events.as_mut_ptr(), maxevents, timeout) };
    if res < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(res)
    }
}

fn eventfd(initva: u32, flags: i32) -> io::Result<i32> {
    let res = unsafe { ffi::eventfd(initva, flags) };
    if res < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(res)
    }
}
