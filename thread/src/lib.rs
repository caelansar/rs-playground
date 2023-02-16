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
    use super::*;

    #[test]
    fn thread_should_works() {
        thread_spawn();
        thread_scope();
    }
}
