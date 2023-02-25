use super::ffi::Kevent;
use super::ffi::Timespec;
use crate::{Events, Interests, Token};
use std::io::{self, IoSliceMut, Read, Write};
use std::net;
use std::os::unix::io::{AsRawFd, RawFd};
use std::ptr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub type Source = std::os::unix::io::RawFd;

pub struct Registrator {
    kq: Source,
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
            let event = Event::new_read_event(fd, token as u64);
            let event = [event];
            kevent(self.kq, &event, &mut [], 0, None)?;
        };

        if interests.is_writable() {
            let event = Event::new_write_event(fd, token as u64);
            let event = [event];
            kevent(self.kq, &event, &mut [], 0, None)?;
        }

        Ok(())
    }

    pub fn unregister(&self, token: usize) -> io::Result<()> {
        todo!()
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
        let event = Event::new_wakeup_event();
        let event = [event];
        kevent(self.kq, &event, &mut [], 0, None)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Selector {
    kq: Source,
}

impl Selector {
    pub fn new() -> io::Result<Self> {
        Ok(Selector { kq: kqueue()? })
    }

    /// This function blocks and waits until an event has been recieved. It never times out.
    pub fn select(&self, events: &mut Events, timeout_ms: Option<i32>) -> io::Result<()> {
        // TODO: get n_events from self
        let n_events = events.capacity() as i32;
        events.clear();
        kevent(self.kq, &[], events, n_events, timeout_ms).map(|n_events| {
            // This is safe because `syscall_kevent` ensures that `n_events` are
            // assigned. We could check for a valid token for each event to verify so this is
            // just a performance optimization used in `mio` and copied here.
            unsafe { events.set_len(n_events as usize) };
        })
    }

    pub fn registrator(&self, is_poll_dead: Arc<AtomicBool>) -> Registrator {
        Registrator {
            kq: self.kq,
            is_poll_dead,
        }
    }
}

impl Drop for Selector {
    fn drop(&mut self) {
        match close(self.kq) {
            Ok(..) => (),
            Err(e) => {
                if !std::thread::panicking() {
                    panic!("{}", e);
                }
            }
        }
    }
}

pub type Event = Kevent;
impl Event {
    pub fn id(&self) -> Token {
        self.udata as usize
    }
}

pub struct TcpStream {
    inner: net::TcpStream,
}

impl TcpStream {
    pub fn connect(adr: impl net::ToSocketAddrs) -> io::Result<Self> {
        // actually we should set this to non-blocking before we call connect which is not something
        // we get from the stdlib but could do with a syscall. Let's skip that step in this example.
        // In other words this will block shortly establishing a connection to the remote server
        let stream = net::TcpStream::connect(adr)?;
        stream.set_nonblocking(true)?;

        Ok(TcpStream { inner: stream })
    }
}

impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // If we let the socket operate non-blocking we could get an error of kind `WouldBlock`,
        // that means there is more data to read but we would block if we waited for it to arrive.
        // The right thing to do is to re-register the event, getting notified once more
        // data is available. We'll not do that in our implementation since we're making an example
        // and instead we make the socket blocking again while we read from it
        self.inner.set_nonblocking(false)?;
        (&self.inner).read(buf)
    }

    /// Copies data to fill each buffer in order, with the final buffer possibly only beeing
    /// partially filled. Now as we'll see this is like it's made for our use case when abstracting
    /// over IOCP AND epoll/kqueue (since we need to buffer anyways).
    ///
    /// IoSliceMut is like `&mut [u8]` but it's guaranteed to be ABI compatible with the `iovec`
    /// type on unix platforms and `WSABUF` on Windows. Perfect for us.
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut]) -> io::Result<usize> {
        (&self.inner).read_vectored(bufs)
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl AsRawFd for TcpStream {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

pub fn kqueue() -> io::Result<i32> {
    let fd = unsafe { super::ffi::kqueue() };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(fd)
}

pub fn kevent(
    kq: RawFd,
    cl: &[Kevent],
    el: &mut [Kevent],
    n_events: i32,
    timeout_ms: Option<i32>,
) -> io::Result<usize> {
    let res = unsafe {
        let kq = kq as i32;
        let cl_len = cl.len() as i32;

        let timeout = timeout_ms.map(Timespec::from_millis);

        let timeout: *const Timespec = match &timeout {
            Some(n) => n,
            None => ptr::null(),
        };

        super::ffi::kevent(kq, cl.as_ptr(), cl_len, el.as_mut_ptr(), n_events, timeout)
    };
    if res < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(res as usize)
}

pub fn close(fd: RawFd) -> io::Result<()> {
    let res = unsafe { super::ffi::close(fd) };
    if res < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Interests;
    use super::*;
    #[test]
    fn create_kevent_works() {
        let selector = Selector::new().unwrap();
        let mut sock = TcpStream::connect("www.baidu.com:80").unwrap();
        let poll_is_dead = Arc::new(AtomicBool::new(false));
        let registrator = selector.registrator(poll_is_dead.clone());

        registrator
            .register(&mut sock, 1, Interests::READABLE)
            .unwrap();
    }

    #[test]
    fn select_kevent_works() {
        let selector = Selector::new().unwrap();
        let mut sock: TcpStream = TcpStream::connect("www.baidu.com:80").unwrap();
        let request = "GET / HTTP/1.1\r\n\
                       Host: www.baidu.com\r\n\
                       Connection: close\r\n\
                       \r\n";
        sock.write_all(request.as_bytes())
            .expect("Error writing to stream");
        let poll_is_dead = Arc::new(AtomicBool::new(false));
        let registrator = selector.registrator(poll_is_dead.clone());

        registrator
            .register(&sock, 99, Interests::READABLE)
            .unwrap();

        let mut events = vec![Event::zero()];

        selector
            .select(&mut events, None)
            .expect("waiting for event.");

        assert_eq!(events[0].udata, 99);
    }

    #[test]
    fn read_kevent_works() {
        let selector = Selector::new().unwrap();
        let mut sock: TcpStream = TcpStream::connect("www.baidu.com:80").unwrap();
        let request = "GET / HTTP/1.1\r\n\
                       Host: www.baidu.com\r\n\
                       Connection: close\r\n\
                       \r\n";
        sock.write_all(request.as_bytes())
            .expect("Error writing to stream");

        let poll_is_dead = Arc::new(AtomicBool::new(false));
        let registrator = selector.registrator(poll_is_dead.clone());

        registrator
            .register(&sock, 100, Interests::READABLE)
            .unwrap();

        let mut events = vec![Event::zero()];

        selector
            .select(&mut events, None)
            .expect("waiting for event.");

        let mut buff = String::new();
        assert!(buff.is_empty());
        sock.read_to_string(&mut buff).expect("Reading to string.");

        assert_eq!(events[0].udata, 100);
        println!("{}", &buff);
        assert!(!buff.is_empty());
    }

    #[test]
    fn write_read_kevent_works() {
        let selector = Selector::new().unwrap();
        let mut sock: TcpStream = TcpStream::connect("www.baidu.com:80").unwrap();

        let poll_is_dead = Arc::new(AtomicBool::new(false));
        let registrator = selector.registrator(poll_is_dead.clone());

        registrator
            .register(&sock, 99, Interests::WRITABLE)
            .unwrap();

        let mut events = vec![Event::zero()];

        selector
            .select(&mut events, None)
            .expect("waiting for evnet.");

        assert_eq!(events[0].udata, 99);

        // sock is writable
        let request = "GET / HTTP/1.1\r\n\
                       Host: www.baidu.com\r\n\
                       Connection: close\r\n\
                       \r\n";
        sock.write_all(request.as_bytes())
            .expect("Error writing to stream");

        // TODO: remove writable interest in kqueue, since we do not interest it anymore

        registrator
            .register(&sock, 100, Interests::READABLE)
            .unwrap();

        let mut events = vec![Event::zero()];

        selector
            .select(&mut events, None)
            .expect("waiting for event.");

        let mut buff = String::new();
        assert!(buff.is_empty());
        sock.read_to_string(&mut buff).expect("Reading to string.");

        assert_eq!(events[0].udata, 100);
        println!("{}", &buff);
        assert!(!buff.is_empty());
    }
}
