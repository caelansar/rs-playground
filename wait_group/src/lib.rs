use std::sync::{Arc, Condvar, Mutex};

#[derive(Clone)]
pub struct WaitGroup {
    count: Arc<Mutex<usize>>,
    cond: Arc<Condvar>,
}

/// A WaitGroup waits for a collection of threads to finish. The main thread calls `add` to set
/// the number of thread to wait for. Then each of the threads runs and calls `done` when finished.
/// At the same time, `wait` can be used to block until all threads have finished.
impl WaitGroup {
    pub fn new() -> Self {
        Self {
            count: Arc::new(Mutex::new(0)),
            cond: Arc::new(Condvar::new()),
        }
    }

    pub fn add(&self, num: usize) {
        *self.count.lock().unwrap() += num
    }

    pub fn done(&self) {
        let mut count = self.count.lock().unwrap();
        *count -= 1;
        let curr = *count;
        drop(count);

        if curr == 0 {
            self.cond.notify_one();
        }
    }

    pub fn wait(&self) {
        let mut count = self.count.lock().unwrap();

        while *count != 0 {
            count = self.cond.wait(count).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use super::*;

    #[test]
    fn test_wait_group() {
        let wg = WaitGroup::new();

        wg.add(5);

        for i in 0..5 {
            let wg = wg.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_secs(2));
                println!("task {} done", i);
                wg.done();
            });
        }
        wg.wait();
        println!("all done");
    }
}
