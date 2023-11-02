use std::alloc::{alloc, dealloc, handle_alloc_error, realloc, Layout};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::{ptr, slice};

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
            unsafe { Some(ptr::read(self.ptr.as_ptr().add(self.len))) }
        }
    }

    pub fn insert(&mut self, elem: T, idx: usize) {
        assert!(idx <= self.len);
        if self.cap == self.len {
            self.grow();
        }
        unsafe {
            ptr::copy(
                self.ptr.as_ptr().add(idx),
                self.ptr.as_ptr().add(idx + 1),
                self.len - idx,
            );
            ptr::write(self.ptr.as_ptr().add(idx), elem);
        }
        self.len += 1;
    }

    pub fn remove(&mut self, idx: usize) -> Option<T> {
        assert!(idx <= self.len);

        self.len -= 1;

        unsafe {
            let rv = unsafe { Some(ptr::read(self.ptr.as_ptr().add(idx))) };
            ptr::copy(
                self.ptr.as_ptr().add(idx + 1),
                self.ptr.as_ptr().add(idx),
                self.len - idx,
            );
            rv
        }
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        if self.cap == 0 {
            return;
        }
        while let Some(_) = self.pop() {}

        let layout = Layout::array::<T>(self.cap).unwrap();
        unsafe { dealloc(self.ptr.as_ptr() as *mut u8, layout) }
    }
}

impl<T> Deref for Vec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
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

    #[test]
    fn vec_deref_works() {
        let mut vec = MyVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        let rv = vec.iter().map(|x| *x).collect::<Vec<i32>>();
        assert_eq!(rv, vec![1, 2, 3]);
    }

    #[test]
    fn vec_insert_remove_works() {
        let mut vec = MyVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        vec.insert(0, 0);
        let rv = vec.iter().map(|x| *x).collect::<Vec<i32>>();
        assert_eq!(rv, vec![0, 1, 2, 3]);

        vec.remove(0);
        let rv = vec.iter().map(|x| *x).collect::<Vec<i32>>();
        assert_eq!(rv, vec![1, 2, 3]);
    }
}
