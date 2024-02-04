mod mutex_poison;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

fn thread_spawn() {
    let name = "cae".to_owned();

    let handler = thread::spawn(move || println!("hello {}", name));
    handler.join().unwrap();
}

fn thread_scope() {
    let name = "cae".to_owned();

    thread::scope(|s| {
        s.spawn(|| println!("hello1 {}", name));
        s.spawn(|| println!("hello2 {}", name));
    });
}

#[derive(Clone)]
pub struct SharedReceiver<T>(Arc<Mutex<Receiver<T>>>);

pub fn shared_channel<T>() -> (Sender<T>, SharedReceiver<T>) {
    let (send, recv) = channel();
    (send, SharedReceiver(Arc::new(Mutex::new(recv))))
}

impl<T> Iterator for SharedReceiver<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.lock().map(|x| x.recv().ok()).unwrap()
    }
}

fn partition<T: PartialOrd + Send>(v: &mut [T]) -> usize {
    let pivot = v.len() - 1;
    let mut i = 0;
    for j in 0..pivot {
        if v[j] <= v[pivot] {
            v.swap(i, j);
            i += 1;
        }
    }
    v.swap(i, pivot);
    i
}

fn quick_sort_rayon<T: PartialOrd + Send>(v: &mut [T]) {
    if v.len() > 1 {
        let mid = partition(v);
        let (lo, hi) = v.split_at_mut(mid);
        rayon::join(|| quick_sort(lo), || quick_sort(hi));
    }
}

fn quick_sort<T: PartialOrd + Send>(v: &mut [T]) {
    if v.len() > 1 {
        let mid = partition(v);
        let (lo, hi) = v.split_at_mut(mid);
        quick_sort(lo);
        quick_sort(hi);
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        time::{Duration, Instant},
    };

    use super::*;

    #[test]
    fn thread_should_works() {
        thread_spawn();
        thread_scope();
    }

    #[test]
    fn thread_concurrency() {
        let data = Arc::new(Mutex::new(1));
        let data1 = data.clone();

        let handler = thread::spawn(move || {
            thread::sleep(Duration::from_secs(1));
            assert_eq!(100, *data.lock().unwrap())
        });

        *data1.lock().unwrap() = 100;
        handler.join().unwrap();
    }

    #[test]
    fn slow_drop_lock() {
        let n = Mutex::new(0);
        thread::scope(|s| {
            for _ in 0..10 {
                s.spawn(|| {
                    let mut guard = n.lock().unwrap();
                    for _ in 0..100 {
                        *guard += 1;
                    }
                    thread::sleep(Duration::from_secs(1));
                });
            }
        });
        assert_eq!(n.into_inner().unwrap(), 1000);
    }

    #[test]
    fn fast_drop_lock() {
        let n = Mutex::new(0);
        thread::scope(|s| {
            for _ in 0..10 {
                s.spawn(|| {
                    let mut guard = n.lock().unwrap();
                    for _ in 0..100 {
                        *guard += 1;
                    }
                    drop(guard); // drop in advance
                    thread::sleep(Duration::from_secs(1));
                });
            }
        });
        assert_eq!(n.into_inner().unwrap(), 1000);
    }

    use loom::sync::{atomic::AtomicUsize, atomic::Ordering};

    #[test]
    fn concurrent_inc_work() {
        loom::model(|| {
            let num = loom::sync::Arc::new(AtomicUsize::new(0));

            let ths: Vec<_> = (0..2)
                .map(|_| {
                    let num = num.clone();
                    loom::thread::spawn(move || {
                        num.fetch_add(1, Ordering::AcqRel);
                    })
                })
                .collect();

            for th in ths {
                th.join().unwrap();
            }

            assert_eq!(2, num.load(Ordering::Relaxed));
        });
    }

    #[test]
    #[should_panic]
    fn concurrent_inc_failed() {
        loom::model(|| {
            let num = loom::sync::Arc::new(AtomicUsize::new(0));

            let ths: Vec<_> = (0..2)
                .map(|_| {
                    let num = num.clone();
                    loom::thread::spawn(move || {
                        let curr = num.load(Ordering::Acquire);
                        num.store(curr + 1, Ordering::Release);
                    })
                })
                .collect();

            for th in ths {
                th.join().unwrap();
            }

            assert_eq!(2, num.load(Ordering::Relaxed));
        });
    }

    #[test]
    fn release_acquire() {
        use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
        use std::sync::atomic::{AtomicBool, AtomicU64};

        let data: &'static AtomicU64 = Box::leak(Box::new(AtomicU64::new(0)));
        let ready: &'static AtomicBool = Box::leak(Box::new(AtomicBool::new(false)));

        thread::spawn(|| {
            data.store(123, Relaxed);
            ready.store(true, Release); // Everything from before this store ..
        });
        while !ready.load(Acquire) {
            // .. is visible after this loads `true`.
            thread::sleep(Duration::from_millis(100));
        }
        assert_eq!(data.load(Relaxed), 123);
    }

    #[test]
    fn test_scope_and_move() {
        let mut a = 0;

        thread::scope(|s| {
            s.spawn(|| {
                a += 1;
            });
        });

        thread::scope(|s| {
            s.spawn(|| {
                a += 1;
            });
        });

        assert_eq!(a, 2);

        let mut a = 0;

        let t1 = thread::spawn(move || {
            a += 1; //copy
        });

        let t2 = thread::spawn(move || {
            a += 1; //copy
        });

        t1.join().unwrap();
        t2.join().unwrap();

        assert_eq!(a, 0);
    }

    #[test]
    fn test_panic_thread() {
        let h1 = thread::spawn(|| panic!("panic in thread"));

        println!("main thread");
        let r = h1.join();
        assert!(r.is_err());
    }

    #[test]
    fn test_shared_receiver() {
        let (tx, mut rx) = shared_channel();

        let tx1 = tx.clone();
        let tx2 = tx.clone();

        let mut rx1 = rx.clone();

        let h1 = thread::spawn(move || {
            tx.send(1).unwrap();
        });
        let h2 = thread::spawn(move || {
            tx1.send(2).unwrap();
        });
        let h3 = thread::spawn(move || {
            tx2.send(3).unwrap();
        });
        let h4 = thread::spawn(move || {
            println!("rx {:?}", rx.next());
            println!("rx {:?}", rx.next());
        });
        let h5 = thread::spawn(move || {
            println!("rx1 {:?}", rx1.next());
            println!("rx1 {:?}", rx1.next());
        });

        h1.join().unwrap();
        h2.join().unwrap();
        h3.join().unwrap();
        h4.join().unwrap();
        h5.join().unwrap();
    }

    #[test]
    fn test_barrier() {
        use std::sync::{Arc, Barrier};
        use std::thread;

        let n = 10;
        let mut handles = Vec::with_capacity(n);
        let barrier = Arc::new(Barrier::new(n));
        for _ in 0..n {
            let c = Arc::clone(&barrier);
            // The same messages will be printed together.
            // You will NOT see any interleaving.
            handles.push(thread::spawn(move || {
                println!("before wait");
                c.wait();
                println!("after wait");
            }));
        }
        // Wait for other threads to finish.
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_rayon_sort() {
        let mut v: Vec<u32> = (1..=10000).into_iter().collect();
        let mut v1 = v.clone();
        v1.swap(11, 45);
        v1.swap(53, 9438);
        v1.swap(49, 898);
        v1.swap(1, 4);

        let start = Instant::now();
        quick_sort(&mut v1);
        println!("{:?}", start.elapsed());

        assert_eq!(v1, v);

        let mut v: Vec<u32> = (1..=10000).into_iter().collect();
        let mut v1 = v.clone();
        v1.swap(11, 45);
        v1.swap(53, 9438);
        v1.swap(49, 898);
        v1.swap(1, 4);

        let start = Instant::now();
        quick_sort_rayon(&mut v1);
        println!("{:?}", start.elapsed());

        assert_eq!(v1, v);
    }
}
