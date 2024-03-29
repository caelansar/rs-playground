use network::cvt;
use std::os::unix::io::RawFd;
use std::time::Duration;
use std::{io, mem, ptr, time};

pub struct FdSet(libc::fd_set);

impl FdSet {
    pub fn new() -> FdSet {
        unsafe {
            // https://doc.rust-lang.org/std/mem/union.MaybeUninit.html#out-pointers
            let mut raw_fd_set = mem::MaybeUninit::<libc::fd_set>::uninit();
            libc::FD_ZERO(raw_fd_set.as_mut_ptr());
            FdSet(raw_fd_set.assume_init())
        }
    }
    pub fn clear(&mut self, fd: RawFd) {
        unsafe { libc::FD_CLR(fd, &mut self.0) }
    }
    pub fn set(&mut self, fd: RawFd) {
        unsafe { libc::FD_SET(fd, &mut self.0) }
    }
    pub fn is_set(&mut self, fd: RawFd) -> bool {
        unsafe { libc::FD_ISSET(fd, &mut self.0) }
    }
}

fn to_fdset_ptr(opt: Option<&mut FdSet>) -> *mut libc::fd_set {
    match opt {
        None => ptr::null_mut(),
        Some(&mut FdSet(ref mut raw_fd_set)) => raw_fd_set,
    }
}
fn to_ptr<T>(opt: Option<&T>) -> *const T {
    match opt {
        None => ptr::null::<T>(),
        Some(p) => p,
    }
}

pub fn select(
    nfds: libc::c_int,
    readfds: Option<&mut FdSet>,
    writefds: Option<&mut FdSet>,
    errorfds: Option<&mut FdSet>,
    timeout: Option<&libc::timeval>,
) -> io::Result<i32> {
    cvt!(libc::select(
        nfds,
        to_fdset_ptr(readfds),
        to_fdset_ptr(writefds),
        to_fdset_ptr(errorfds),
        to_ptr::<libc::timeval>(timeout) as *mut libc::timeval,
    ))
}

pub fn new_timeval(duration: Duration) -> libc::timeval {
    libc::timeval {
        tv_sec: duration.as_secs() as i64,
        #[cfg(target_os = "macos")]
        tv_usec: duration.subsec_micros() as i32,
        #[cfg(target_os = "linux")]
        tv_usec: duration.subsec_micros() as i64,
    }
}

pub fn main() {
    let mut reads = FdSet::new();

    loop {
        let timeout = new_timeval(Duration::from_secs(5));

        reads.set(0); // standard input
        match select(1, Some(&mut reads), None, None, Some(&timeout)) {
            Ok(result) => {
                if result == 0 {
                    println!("timeout");
                }
                if reads.is_set(0) {
                    let mut buf = Vec::with_capacity(1024);
                    let n = cvt!(libc::read(0, buf.as_mut_ptr(), 30)).unwrap();
                    let slice = unsafe {
                        std::slice::from_raw_parts(buf.as_ptr() as *const u8, n as usize)
                    };
                    println!(
                        "read {n} bytes from console, data: {}",
                        String::from_utf8_lossy(slice)
                    );
                }
            }
            Err(e) => {
                println!("select err: {:?}", e)
            }
        }
    }
}
