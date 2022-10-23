#[cfg(test)]
mod tests {
    use mio::net::TcpStream;
    use mio::{Events, Interest, Poll, Token};
    use std::collections::HashMap;
    use std::error::Error;
    use std::io::{self, Read, Write};

    #[cfg(test)]
    fn register(
        sockets: &mut HashMap<Token, TcpStream>,
        poll: &Poll,
    ) -> Result<(), Box<dyn Error>> {
        let addr = "127.0.0.1:5000".parse()?;

        for i in 0..10 {
            // Setup the client socket.
            let mut client = TcpStream::connect(addr)?;

            // Register the socket.
            poll.registry().register(
                &mut client,
                Token(i),
                Interest::READABLE.add(Interest::WRITABLE),
            )?;

            sockets.insert(Token(i), client);
        }
        Ok(())
    }

    #[test]
    fn mio_should_works() -> Result<(), Box<dyn Error>> {
        let mut sockets: HashMap<Token, TcpStream> = HashMap::new();
        let mut w: HashMap<Token, bool> = HashMap::new();
        // Create a poll instance.
        let mut poll = Poll::new()?;
        // Create storage for events.
        let mut events = Events::with_capacity(128);

        register(&mut sockets, &poll)?;

        let mut buffer = [0u8; 1024];

        // Start an event loop.
        loop {
            if sockets.len() == 0 {
                println!("All tasks done");
                return Ok(());
            }
            // Poll Mio for events, blocking until we get an event.
            poll.poll(&mut events, None)?;

            // Process each event.
            for event in events.iter() {
                // We can use the token we previously provided to `register` to
                // determine for which socket the event is.
                match event.token().0 {
                    0..=9 => {
                        let token = event.token();
                        if event.is_writable() {
                            // If socket has not been written before
                            if *(w.entry(token).or_insert(false)) {
                                println!("Already write");
                            } else {
                                let req = "GET / HTTP/1.1\r\nHost: 127.0.0.1:5000\r\n\r\n";
                                println!("Write data for token {}", token.0);
                                // Write to the socket without blocking.
                                sockets.get_mut(&token).unwrap().write_all(req.as_bytes())?;
                                w.insert(token, true);
                            }
                        }

                        if event.is_readable() {
                            // Read from the socket without blocking.
                            let read = sockets.get_mut(&token).unwrap().read(&mut buffer);
                            match read {
                                Ok(len) => {
                                    // Now do something with &buffer[0..len]
                                    println!(
                                        "Read {} bytes for token {}, bytes: {:?}",
                                        len,
                                        token.0,
                                        &buffer[0..len]
                                    );
                                }
                                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                                Err(e) => panic!("Unexpected error: {}", e),
                            }
                            sockets.remove(&token);
                        }
                    }
                    // We don't expect any events with tokens other than those we provided.
                    _ => unreachable!(),
                }
            }
        }
    }
}
