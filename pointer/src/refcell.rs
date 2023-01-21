use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

use crate::cell::Cell;

#[derive(Clone, Copy)]
enum State {
    Unshared,
    Shared(usize),
    Exclusive,
}

pub struct RefCell<T> {
    data: UnsafeCell<T>,
    state: Cell<State>,
}

impl<T> RefCell<T> {
    pub fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
            state: Cell::new(State::Unshared),
        }
    }

    pub fn borrow(&self) -> Option<Ref<'_, T>> {
        match self.state.get() {
            State::Unshared => {
                self.state.set(State::Shared(1));
                Some(Ref { refcell: self })
            }
            State::Shared(n) => {
                self.state.set(State::Shared(n + 1));
                Some(Ref { refcell: self })
            }
            State::Exclusive => None,
        }
    }

    pub fn borrow_mut(&self) -> Option<RefMut<'_, T>> {
        match self.state.get() {
            State::Shared(_) | State::Exclusive => None,
            State::Unshared => {
                self.state.set(State::Exclusive);
                Some(RefMut { refcell: self })
            }
        }
    }
}

pub struct Ref<'a, T> {
    refcell: &'a RefCell<T>,
}

impl<T> Drop for Ref<'_, T> {
    fn drop(&mut self) {
        match self.refcell.state.get() {
            State::Unshared | State::Exclusive => unreachable!(),
            State::Shared(1) => self.refcell.state.set(State::Unshared),

            State::Shared(n) => self.refcell.state.set(State::Shared(n - 1)),
        }
    }
}

impl<T> Deref for Ref<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.refcell.data.get() }
    }
}

pub struct RefMut<'a, T> {
    refcell: &'a RefCell<T>,
}

impl<T> Drop for RefMut<'_, T> {
    fn drop(&mut self) {
        match self.refcell.state.get() {
            State::Unshared | State::Shared(_) => unreachable!(),
            State::Exclusive => self.refcell.state.set(State::Unshared),
        }
    }
}

impl<T> Deref for RefMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.refcell.data.get() }
    }
}

impl<T> DerefMut for RefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.refcell.data.get() }
    }
}
