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
}
