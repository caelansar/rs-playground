use std::io::{IoSliceMut, Read, Write};
use std::os::fd::{AsRawFd, RawFd};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use std::{io, net};

#[cfg(target_os = "linux")]
mod linux_poll;

#[cfg(target_os = "linux")]
pub use linux_poll::epoll::{Registrator, Selector};

#[cfg(target_os = "macos")]
mod macos_poll;

#[cfg(target_os = "macos")]
pub use macos_poll::kqueue::{KeventList, Registrator, Selector};

#[cfg(target_os = "macos")]
pub type Events = KeventList;

#[cfg(target_os = "linux")]
pub type Events = linux_poll::epoll::Events;

pub type Token = usize;

#[macro_export]
macro_rules! cvt {
    ($libc_call: expr) => {
        cvt::cvt(unsafe { $libc_call })
    };
}

/// `Poll` represents the event queue. The `poll` method will block the current thread
/// waiting for events. If no timeout is provided it will potentially block indefinately.
///
/// `Poll` can be used in one of two ways. The first way is by registering interest in events and then wait for
/// them in the same thread. In this case you'll use the built-in methods on `Poll` for registering events.
///
/// Alternatively, it can be used by waiting in one thread and registering interest in events from
/// another. In this case you'll need to call the `Poll::registrator()` method which returns a `Registrator`
/// tied to this event queue which can be sent to another thread and used to register events.
#[derive(Debug)]
pub struct Poll {
    registry: Registry,
    is_poll_dead: Arc<AtomicBool>,
}

impl Poll {
    pub fn new() -> io::Result<Poll> {
        Selector::new().map(|selector| Poll {
            registry: Registry { selector },
            is_poll_dead: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn registrator(&self) -> Registrator {
        self.registry
            .selector
            .registrator(self.is_poll_dead.clone())
    }

    /// Polls the event loop. The thread yields to the OS while witing for either
    /// an event to retur or a timeout to occur.
    pub fn poll(&mut self, events: &mut Events, timeout: Option<Duration>) -> io::Result<usize> {
        loop {
            let res = self.registry.selector.select(events, timeout);
            match res {
                Ok(()) => break,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => (),
                Err(e) => return Err(e),
            };
        }

        if self.is_poll_dead.load(Ordering::SeqCst) {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "Poll closed."));
        }

        Ok(events.len())
    }
}

#[derive(Debug)]
pub struct Registry {
    selector: Selector,
}

const WRITABLE: u8 = 0b0000_0001;
const READABLE: u8 = 0b0000_0010;

/// Represents interest in either Read or Write events. This struct is created
/// by using one of the two constants:
///
/// - Interests::READABLE
/// - Interests::WRITABLE
pub struct Interests(u8);
impl Interests {
    pub const READABLE: Interests = Interests(READABLE);
    pub const WRITABLE: Interests = Interests(WRITABLE);

    pub fn is_readable(&self) -> bool {
        self.0 & READABLE != 0
    }

    pub fn is_writable(&self) -> bool {
        self.0 & WRITABLE != 0
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

trait EventID {
    fn id(&self) -> Token;
}

#[cfg(test)]
mod tests {
    use std::{
        io::{self, Write},
        sync::mpsc::{self, SyncSender},
        thread::{self, sleep, JoinHandle},
        time::Duration,
    };

    use crate::{EventID, Events, Interests, Poll, Registrator, TcpStream};

    struct Reactor {
        handle: Option<JoinHandle<()>>,
        register: Option<Registrator>,
    }

    impl Reactor {
        fn new(sender: SyncSender<usize>) -> Reactor {
            let mut poll = Poll::new().unwrap();
            let registrator = poll.registrator();

            // Set up the kqueue/epoll/IOCP event loop in a separated thread
            let handle = thread::spawn(move || {
                let mut events = Events::with_capacity(1024);
                loop {
                    println!("waiting {:?}", poll);
                    match poll.poll(&mut events, Some(Duration::from_millis(100))) {
                        Ok(n) => println!("recv {n} events"),
                        Err(ref e) if e.kind() == io::ErrorKind::Interrupted => break,
                        Err(e) => panic!("Poll error: {:?}, {}", e.kind(), e),
                    };

                    println!("events num: {}", events.len());
                    for event in &events {
                        let event_token = event.id();
                        println!("send token: {}", event_token);
                        sender.send(event_token).expect("Send event_token err.");
                    }
                }
            });

            Reactor {
                handle: Some(handle),
                register: Some(registrator),
            }
        }

        fn registrator(&mut self) -> Registrator {
            self.register.take().unwrap()
        }
    }

    impl Drop for Reactor {
        fn drop(&mut self) {
            self.handle.take().map(|h| h.join().unwrap());
        }
    }

    #[test]
    fn reactor_works() {
        let (sender, receiver) = mpsc::sync_channel(1);
        let mut reactor = Reactor::new(sender);
        let registrator = reactor.registrator();

        let mut socket = TcpStream::connect("www.baidu.com:80").unwrap();
        let request = "GET / HTTP/1.1\r\n\
                       Host: www.baidu.com\r\n\
                       Connection: close\r\n\
                       \r\n";
        socket
            .write_all(request.as_bytes())
            .expect("Error writing to stream");

        registrator
            .register(&socket, 99, Interests::READABLE)
            .unwrap();

        thread::Builder::new()
            .name("wait".to_string())
            .spawn(move || {
                sleep(Duration::from_millis(1000));
                registrator.close_loop().unwrap();
            })
            .unwrap();

        while let Ok(token) = receiver.recv() {
            println!("receive token");
            assert_eq!(99, token);
        }
    }
}
