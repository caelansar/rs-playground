use std::io;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

#[cfg(target_os = "macos")]
mod macos_poll;
use macos_poll::kqueue::KeventList;
pub use macos_poll::kqueue::{Registrator, Selector, TcpStream};

#[cfg(target_os = "macos")]
pub type Events = KeventList;
pub type Token = usize;

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
