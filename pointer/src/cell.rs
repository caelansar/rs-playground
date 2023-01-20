use std::{cell::UnsafeCell, ops::Deref};

pub struct Cell<T> {
    data: UnsafeCell<T>,
}

impl<T> Cell<T> {
    pub const fn new(data: T) -> Self {
        Cell {
            data: UnsafeCell::new(data),
        }
    }

    pub fn set(&self, data: T) {
        unsafe { *self.data.get() = data }
    }

    pub fn get(&self) -> T
    where
        T: Copy,
    {
        unsafe { *self.data.get() }
    }
}

impl<T> Deref for Cell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data.get() }
    }
}
