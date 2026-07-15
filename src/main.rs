use rustwebserver::{CONFIG, LISTENER, GlobalConfig, Server, ThreadPool};

use mio::{
    Events, Interest, Poll, net::TcpListener,
};

use std::{
    io::ErrorKind,
    sync::{
        Arc,
        Mutex,
    }, thread
};

fn main() {
    let args: Vec<String> = std::env::args().collect::<Vec<String>>();
    if args.len() == 1 {
        println!("usage: {} <config>", args[0]);
        return;
    }

    match CONFIG.set(GlobalConfig::new(args)) {
        Ok(_) => (),
        Err(_) => panic!("Failed to setup config"),
    }

    //println!("{CONFIG:#?}");

    let names: Vec<&String> = CONFIG.get().unwrap().servers.keys().collect();

    let pool = ThreadPool::new(names.len());

    for name in names {
        pool.execute(move || {
           start(name.clone());
        });
    }
}


pub fn start(name: String) {
    
    let mut listener =
        match TcpListener::bind(CONFIG.get().unwrap().servers.get(&name).unwrap().addr) {
            Ok(listener) => listener,
            Err(error) => panic!("Problem binding TcpListener: {error:?}"),
        };

    let poll = match Poll::new() {
        Ok(p) => Arc::new(Mutex::new(p)),
        Err(error) => panic!("Failed to create os-poll structure {error:?}"),
    };
    let mut events = Events::with_capacity(256);

    match poll.lock().unwrap()
        .registry()
        .register(&mut listener, LISTENER, Interest::READABLE)
    {
        Ok(_) => (),
        Err(error) => panic!("Failed to register listener: {error:?}"),
    };

    let server = Arc::new(Server::new(name, Arc::new(Mutex::new(listener))));

    loop {
        match poll.lock() {
            Ok(mut p) => {
                //poll_reg = p.registry().clone().into();
                match p.poll(&mut events, None) {
                    Ok(_) => {}
                    Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                    Err(error) => {
                        panic!("Poll failed: {error:?}");
                    }
                }
                drop(p);
        },
            Err(_) => panic!("Couldn't acquire lock"),
        }

        //println!("Got out of poll");

        for event in events.iter() {
            //println!("Got event");
            let cloned_poll_reg = Arc::clone(&poll);
            let cloned_server = Arc::clone(&server);
            let cloned_event = event.clone();
            match event.token() {
                LISTENER => {
                    let handle = thread::spawn(move || {
                        //println!("Attempting to accept connection");
                        cloned_server
                        .accept_new_connection(cloned_poll_reg).expect("Error accepting connection.");
                    });
                    //handles.push(handle);
                    handle.join().unwrap();
                },
                _ => {
                    let handle = thread::spawn(move || {
                        cloned_server
                        .established_connection(cloned_poll_reg, &cloned_event);
                    });
                    //handles.push(handle);
                    handle.join().unwrap();
                },
            }
        }
        /*
        for handle in handles {
            handle.join().unwrap();
        }
        */
    }
}