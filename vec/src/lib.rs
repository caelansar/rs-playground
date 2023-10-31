use std::alloc::{alloc, realloc, Layout, handle_alloc_error};
use std::ptr;
use std::ptr::NonNull;

pub struct Vec<T> {
    ptr: NonNull<T>,
    len: usize,
    cap: usize,
}

impl<T> Vec<T> {
    pub fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            len: 0,
            cap: 0,
        }
    }

    fn grow(&mut self) {
        let (cap, layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1).unwrap())
        } else {
            (self.cap * 2, Layout::array::<T>(self.cap * 2).unwrap())
        };

        assert!(layout.size() <= isize::MAX as usize);

        let ptr = if self.cap == 0 {
            unsafe { alloc(layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_prt = self.ptr.as_ptr();
            unsafe { realloc(old_prt as *mut u8, old_layout, layout.size()) }
        };

        self.ptr = match NonNull::new(ptr as *mut T) {
            Some(p) => p,
            None => handle_alloc_error(layout),
        };
        self.cap = cap;
    }

    pub fn push(&mut self, elem: T) {
        if self.len == self.cap {
            self.grow();
        }
        unsafe {
            ptr::write(self.ptr.as_ptr().add(self.len), elem);
        }

        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe  {
                Some(ptr::read(self.ptr.as_ptr().add(self.len)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Vec as MyVec;

    #[test]
    fn vec_works() {
        let mut vec = MyVec::new();
        vec.push(1);
        assert_eq!(Some(1), vec.pop());
        assert_eq!(None, vec.pop());
    }
}
