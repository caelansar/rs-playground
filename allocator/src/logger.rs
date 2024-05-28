use std::fmt;
use std::fmt::Write;

pub struct Logger;

impl Write for Logger {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // SAFETY: using libc to write to stdout
        unsafe {
            libc::write(1, s.as_ptr() as *const libc::c_void, s.len());
        }
        Ok(())
    }
}

pub fn log_alloc(ptr: *mut u8, size: usize) {
    let mut logger = Logger;
    writeln!(logger, "allocated {} bytes at {:p}", size, ptr).unwrap();
}

pub fn log_dealloc(ptr: *mut u8, size: usize) {
    let mut logger = Logger;
    writeln!(logger, "deallocated {} bytes at {:p}", size, ptr).unwrap();
}
