use std::{
    cell::UnsafeCell,
    sync::atomic::{AtomicBool, Ordering},
};

const LOCKED: bool = true;
const UNLOCKED: bool = false;

pub struct Mutex<T> {
    locked: AtomicBool,
    v: UnsafeCell<T>,
}

unsafe impl<T> Sync for Mutex<T> where T: Send {}

impl<T> Mutex<T> {
    pub fn new(t: T) -> Self {
        Self {
            locked: AtomicBool::new(UNLOCKED),
            v: UnsafeCell::new(t),
        }
    }
    pub fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        while self
            .locked
            .compare_exchange_weak(UNLOCKED, LOCKED, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            // wait until unlocked
            while self.locked.load(Ordering::Relaxed) == LOCKED {}
        }
        let ret = f(unsafe { &mut *self.v.get() });
        self.locked.store(UNLOCKED, Ordering::Release);
        ret
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread::spawn};

    use crate::mutex::Mutex;

    #[test]
    fn mutex_should_works() {
        let l = Arc::new(Mutex::new(0));

        let handlers: Vec<_> = (0..100)
            .map(|_| {
                let data = l.clone();
                spawn(move || {
                    for _ in 0..1000 {
                        data.with_lock(|v| *v += 1)
                    }
                })
            })
            .collect();
        for handler in handlers {
            handler.join().unwrap();
        }
        assert_eq!(l.with_lock(|v| *v), 100 * 1000);
    }
}
