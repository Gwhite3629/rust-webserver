use rustwebserver::{CONFIG, LISTENER, GlobalConfig, Server, ThreadPool};

use mio::{
    Events, Interest, Poll,
};

use std::{
    io::ErrorKind,
    sync::{
        Arc,
        Mutex,
    },
    thread,
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
    let server = Arc::new(Mutex::new(Server::new(name)));

    let poll = match Poll::new() {
        Ok(p) => Arc::new(Mutex::new(p)),
        Err(error) => panic!("Failed to create os-poll structure {error:?}"),
    };
    let mut events = Events::with_capacity(256);

    match poll.try_lock().unwrap()
        .registry()
        .register(&mut server.lock().unwrap().listener, LISTENER, Interest::READABLE)
    {
        Ok(_) => (),
        Err(error) => panic!("Failed to register listener: {error:?}"),
    };

    loop {
        match poll.lock().unwrap().poll(&mut events, None) {
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(error) => {
                panic!("Poll failed: {error:?}");
            }
        }

        //let mut handles = vec![];

        for event in events.iter() {
            //println!("Got event");
            let cloned_poll = Arc::clone(&poll);
            let cloned_server = Arc::clone(&server);
            let cloned_event = event.clone();
            match event.token() {
                LISTENER => {
                    let handle = thread::spawn(move || {
                        cloned_server
                        .lock().unwrap()
                        .accept_new_connection(cloned_poll.lock().unwrap().registry()).expect("Error accepting connection.");
                    });
                    //handles.push(handle);
                    handle.join().unwrap();
                },
                _ => {
                    let handle = thread::spawn(move || {
                        cloned_server
                        .lock().unwrap()
                        .established_connection(cloned_poll.lock().unwrap().registry(), &cloned_event);
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