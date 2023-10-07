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

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
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
}
