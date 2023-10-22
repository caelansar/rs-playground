use crate::{cvt, EventID, Events, Interests, TcpStream, Token};
use libc::{self, c_void};
use std::cmp;
use std::io::{self, Read, Write};
use std::os::raw::{c_int, c_short};
use std::os::unix::io::{AsRawFd, RawFd};
use std::ptr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

pub type Source = std::os::unix::io::RawFd;

type Filter = c_short;
type UData = *mut c_void;
type Count = c_int;

pub type KeventList = Vec<libc::kevent>;

impl EventID for libc::kevent {
    fn id(&self) -> Token {
        self.udata as Token
    }
}

pub trait Zero {
    fn zero() -> Self;
}

impl Zero for KeventList {
    fn zero() -> Self {
        vec![libc::kevent {
            ident: 0,
            filter: 0,
            flags: 0,
            fflags: 0,
            data: 0,
            udata: std::ptr::null_mut::<c_void>(),
        }]
    }
}
macro_rules! kevent {
    ($id: expr, $filter: expr, $flags: expr, $data: expr) => {
        libc::kevent {
            ident: $id as ::libc::uintptr_t,
            filter: $filter as Filter,
            flags: $flags,
            fflags: 0,
            data: 0,
            udata: $data as UData,
        }
    };
}

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
            let flags = libc::EV_ADD | libc::EV_ENABLE | libc::EV_ONESHOT;
            let changes = [kevent!(fd, libc::EVFILT_READ, flags, token)];

            cvt!(libc::kevent(
                self.kq,
                changes.as_ptr(),
                changes.len() as Count,
                [].as_mut_ptr(),
                0,
                std::ptr::null(),
            ))?;
        };

        if interests.is_writable() {
            let flags = libc::EV_ADD | libc::EV_ENABLE | libc::EV_ONESHOT;
            let changes = [kevent!(fd, libc::EVFILT_WRITE, flags, token)];
            cvt!(libc::kevent(
                self.kq,
                changes.as_ptr(),
                changes.len() as Count,
                [].as_mut_ptr(),
                0,
                std::ptr::null(),
            ))?;
        }

        Ok(())
    }

    pub fn deregister(&self, stream: &TcpStream) -> io::Result<()> {
        let fd = stream.as_raw_fd();

        let flags = libc::EV_DELETE | libc::EV_RECEIPT;
        let mut changes = [
            kevent!(fd, libc::EVFILT_READ, flags, ptr::null_mut()),
            kevent!(fd, libc::EVFILT_WRITE, flags, ptr::null_mut()),
        ];
        cvt!(libc::kevent(
            self.kq,
            changes.as_ptr(),
            changes.len() as c_int,
            changes.as_mut_ptr(),
            changes.len() as c_int,
            ::std::ptr::null()
        ))?;
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

        let flags = libc::EV_ADD | libc::EV_ENABLE | libc::EV_CLEAR;
        let changes = [kevent!(0, libc::EVFILT_TIMER, flags, 0)];

        cvt!(libc::kevent(
            self.kq,
            changes.as_ptr(),
            changes.len() as Count,
            [].as_mut_ptr(),
            0,
            std::ptr::null(),
        ))?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Selector {
    kq: Source,
}

impl Selector {
    pub fn new() -> io::Result<Self> {
        Ok(Selector {
            kq: cvt!(libc::kqueue()).unwrap(),
        })
    }

    /// This function blocks and waits until an event has been recieved. It never times out.
    pub fn select(&self, events: &mut Events, timeout: Option<Duration>) -> io::Result<()> {
        let timeout = timeout.map(|to| libc::timespec {
            tv_sec: cmp::min(to.as_secs(), libc::time_t::max_value() as u64) as libc::time_t,
            tv_nsec: to.subsec_nanos() as libc::c_long,
        });
        let timeout = timeout
            .as_ref()
            .map(|s| s as *const _)
            .unwrap_or(ptr::null_mut());

        let n_events = events.capacity() as c_int;
        events.clear();

        let cnt = cvt!(libc::kevent(
            self.kq,
            ptr::null(),
            0,
            events.as_mut_ptr(),
            n_events,
            timeout,
        ))?;
        unsafe { events.set_len(cnt as usize) };
        Ok(())
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
        let registrator = selector.registrator(poll_is_dead);

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
        let registrator = selector.registrator(poll_is_dead);

        registrator
            .register(&sock, 99, Interests::READABLE)
            .unwrap();

        let mut events = KeventList::zero();

        selector
            .select(&mut events, None)
            .expect("waiting for event.");

        let udata = events[0].udata;
        assert_eq!(udata, 99 as *mut c_void);
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
        let registrator = selector.registrator(poll_is_dead);

        registrator
            .register(&sock, 100, Interests::READABLE)
            .unwrap();

        let mut events = KeventList::zero();

        selector
            .select(&mut events, None)
            .expect("waiting for event.");

        let mut buff = String::new();
        assert!(buff.is_empty());
        sock.read_to_string(&mut buff).expect("Reading to string.");

        let udata = events[0].udata;
        assert_eq!(udata, 100 as *mut c_void);
        println!("{}", &buff);
        assert!(!buff.is_empty());
    }

    #[test]
    fn write_read_kevent_works() {
        let selector = Selector::new().unwrap();
        let mut sock: TcpStream = TcpStream::connect("www.baidu.com:80").unwrap();

        let poll_is_dead = Arc::new(AtomicBool::new(false));
        let registrator = selector.registrator(poll_is_dead);

        registrator
            .register(&sock, 99, Interests::WRITABLE)
            .unwrap();

        let mut events = KeventList::zero();

        selector
            .select(&mut events, None)
            .expect("waiting for evnet.");

        let udata = events[0].udata;
        assert_eq!(udata, 99 as *mut c_void);

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

        let mut events = KeventList::zero();

        selector
            .select(&mut events, None)
            .expect("waiting for event.");

        let mut buff = String::new();
        assert!(buff.is_empty());
        sock.read_to_string(&mut buff).expect("Reading to string.");

        let udata = events[0].udata;
        assert_eq!(udata, 100 as *mut c_void);
        println!("{}", &buff);
        assert!(!buff.is_empty());
    }
}
