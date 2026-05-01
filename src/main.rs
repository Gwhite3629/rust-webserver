use std::io::ErrorKind;

use mio::{
    Events, Interest, Poll, net::TcpListener
};

use rustwebserver::{
    HttpConfig,
    Server,
    Processor,
    tls_setup,
    CONFIG,
};

const LISTENER: mio::Token = mio::Token(0);

fn main() {

    let args: Vec<String> = std::env::args().collect::<Vec<String>>();
    if args.len() == 1 {
        println!("usage: {} <config>", args[0]);
        return;
    }

    match CONFIG.set(HttpConfig::new(args)) {
        Ok(_) => (),
        Err(_) => panic!("Failed to setup config"),
    }

    let server_config = tls_setup();

    let mut poll = match Poll::new() {
        Ok(p) => p,
        Err(error) => panic!("Failed to create os-poll structure {error:?}"),
    };

    let mut listener = match TcpListener::bind(CONFIG.get().unwrap().addr) {
        Ok(listener) => listener,
        Err(error) => panic!("Problem binding TcpListener: {error:?}"),
    };

    match poll.registry().register(&mut listener, LISTENER, Interest::READABLE) {
        Ok(_) => (),
        Err(error) => panic!("Failed to register listener: {error:?}"),
    };

    let mut server = Server::new(listener, server_config, Processor::HTTP);

    let mut events = Events::with_capacity(256);

    //let pool = ThreadPool::new(4);

    loop {
        match poll.poll(&mut events, None) {
            Ok(_) => {},
            Err(error) if error.kind() == ErrorKind::Interrupted => continue ,
            Err(error) => {
                panic!("Poll failed: {error:?}");
            },
        }

        for event in events.iter() {
            match event.token() {
                LISTENER => server.accept_new_connection(poll.registry()).expect("Error accepting connection."),
                _ => server.established_connection(poll.registry(), event),
            }
        }
    }
}
