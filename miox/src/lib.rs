#[cfg(test)]
mod tests {
    use bytes::BufMut;
    use mio::net::{TcpListener, TcpStream};
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
            poll.registry()
                .register(&mut client, Token(i), Interest::WRITABLE)?;

            sockets.insert(Token(i), client);
        }
        Ok(())
    }

    #[test]
    fn mio_server_should_works() -> Result<(), Box<dyn Error>> {
        let addr = "0.0.0.0:5001".parse()?;
        let mut listener = TcpListener::bind(addr).unwrap();

        // Create a poll instance.
        let mut poll = Poll::new()?;
        // Create storage for events.
        let mut events = Events::with_capacity(128);

        poll.registry()
            .register(&mut listener, Token(0), Interest::READABLE)?;

        let mut counter: usize = 0;
        let mut sockets: HashMap<Token, TcpStream> = HashMap::new();
        let mut request: HashMap<Token, Vec<u8>> = HashMap::new();

        // Start an event loop.
        loop {
            poll.poll(&mut events, None)?;

            for event in &events {
                match event.token() {
                    Token(0) => {
                        loop {
                            match listener.accept() {
                                Ok((mut socket, address)) => {
                                    println!("Got connection from {}", address);

                                    counter += 1;
                                    let token = Token(counter);

                                    // Register for readable events
                                    poll.registry().register(
                                        &mut socket,
                                        token,
                                        Interest::READABLE,
                                    )?;
                                    sockets.insert(token, socket);
                                }
                                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                                Err(e) => panic!("Unexpected error: {}", e),
                            }
                        }
                    }
                    token if event.is_readable() => {
                        let mut buffer = [0 as u8; 1024];
                        let mut req = vec![];
                        loop {
                            let read = sockets.get_mut(&token).unwrap().read(&mut buffer);
                            match read {
                                Ok(0) => {
                                    // Disconnect
                                    poll.registry()
                                        .deregister(sockets.get_mut(&token).unwrap())?;
                                    sockets.remove(&token);
                                    break;
                                }
                                Ok(len) => {
                                    req.put(&buffer[0..len]);
                                    println!("Read {} bytes for token {}", len, token.0);
                                    break;
                                }
                                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                                Err(e) => panic!("Unexpected error: {}", e),
                            }
                        }
                        if sockets.get_mut(&token).is_some() {
                            poll.registry().reregister(
                                sockets.get_mut(&token).unwrap(),
                                token,
                                Interest::WRITABLE,
                            )?;
                            request.insert(token, req);
                        }
                    }
                    token if event.is_writable() => {
                        let req = request.get(&token).unwrap();
                        let socket = sockets.get_mut(&token).unwrap();
                        socket.write_all(req.as_slice()).unwrap();
                        request.remove(&token);
                        poll.registry()
                            .reregister(socket, token, Interest::READABLE)?;
                    }
                    // We don't expect any events with tokens other than those we provided.
                    _ => unreachable!(),
                }
            }
        }
    }

    #[test]
    fn mio_client_should_works() -> Result<(), Box<dyn Error>> {
        let mut sockets: HashMap<Token, TcpStream> = HashMap::new();
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
                            let req = "GET / HTTP/1.1\r\nHost: 127.0.0.1:5000\r\n\r\n";
                            println!("Write data for token {}", token.0);
                            // Write to the socket without blocking.
                            let socket = sockets.get_mut(&token).unwrap();
                            socket.write_all(req.as_bytes())?;
                            poll.registry()
                                .reregister(socket, token, Interest::READABLE)?;
                        }

                        if event.is_readable() {
                            // Read from the socket without blocking.
                            let read = sockets.get_mut(&token).unwrap().read(&mut buffer);
                            match read {
                                Ok(len) => {
                                    println!(
                                        "Read {} bytes for token {}, bytes: {:?}",
                                        len,
                                        token.0,
                                        String::from_utf8_lossy(&buffer[0..len])
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
