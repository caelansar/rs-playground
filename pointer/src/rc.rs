use std::ops::Deref;

pub struct Rc<T> {
    inner: *mut Inner<T>,
}

struct Inner<T> {
    count: usize,
    data: T,
}

impl<T> Rc<T> {
    pub fn new(data: T) -> Self {
        let inner = Inner { count: 1, data };
        Self {
            inner: Box::into_raw(Box::new(inner)),
        }
    }
}

impl<T> Clone for Rc<T> {
    fn clone(&self) -> Self {
        unsafe { &mut *self.inner }.count += 1;
        Self { inner: self.inner }
    }
}

impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        let count = &mut unsafe { &mut *self.inner }.count;
        if *count == 1 {
            let _ = unsafe { Box::from_raw(self.inner) };
        } else {
            *count -= 1;
        }
    }
}

impl<T> Deref for Rc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &unsafe { &*self.inner }.data
    }
}
