#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn test_lock_poison() {
        let mutex = Arc::new(Mutex::new(HashSet::new()));

        // poison the mutex
        let c_mutex = Arc::clone(&mutex);
        let _ = thread::Builder::new()
            .name("locker_poison".to_string())
            .spawn(move || {
                let mut data = c_mutex.lock().unwrap();
                data.insert(10);
                panic!();
            })
            .unwrap()
            .join();

        let err = mutex.lock().unwrap_err();
        let data = err.into_inner();
        println!("recovered HashSet: {:?}", data);
    }
}
