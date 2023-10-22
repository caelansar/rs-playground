use super::ffi::{close_fd, epoll_create, epoll_ctl, epoll_wait};
use crate::{EventID, Interests, TcpStream, Token};
use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::{io, time};

const READ_FLAG: i32 = libc::EPOLLIN | libc::EPOLLONESHOT;

pub type Events = Vec<libc::epoll_event>;

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
            let mut event = libc::epoll_event {
                events: READ_FLAG as u32,
                u64: token as u64,
            };
            epoll_ctl(self.fd, libc::EPOLL_CTL_ADD, fd, &mut event)?;
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

    /// This function blocks and waits until an event has been received. `timeout` None means
    /// the poll will never time out.
    pub fn select(
        &self,
        events: &mut Events,
        timeout_ms: Option<time::Duration>,
    ) -> io::Result<()> {
        events.clear();
        let timeout = timeout_ms.unwrap_or(time::Duration::from_millis(100));
        epoll_wait(self.fd, events, 1024, timeout.as_millis() as i32).map(|n_events| {
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

impl EventID for libc::epoll_event {
    fn id(&self) -> Token {
        self.u64 as Token
    }
}

#[cfg(test)]
mod tests {
    use super::Interests;
    use super::*;
    use std::io::Write;

    #[test]
    fn epoll_works() {
        let selector = Selector::new().unwrap();
        let mut sock: TcpStream = TcpStream::connect("www.baidu.com:80").unwrap();
        let request = "GET / HTTP/1.1\r\n\
                       Host: www.baidu.com\r\n\
                       Connection: close\r\n\
                       \r\n";
        sock.write_all(request.as_bytes())
            .expect("Error writing to stream");
        let poll_is_dead = Arc::new(AtomicBool::new(false));
        let registrator = selector.registrator(poll_is_dead);

        registrator
            .register(&sock, 99, Interests::READABLE)
            .unwrap();

        let mut events = Events::with_capacity(16);

        selector
            .select(&mut events, None)
            .expect("waiting for event.");

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id(), 99);
    }
}
