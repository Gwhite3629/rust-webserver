use std::net::TcpListener;
use std::sync::Arc;

use native_tls::{
    Identity,
    TlsAcceptor,
    TlsStream,
};

use rustwebserver::{
    HttpConfig,
    HttpMethodHandlerTable,
    ThreadPool,
    handle_connection,
    CONFIG,
};

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

    let listener = match TcpListener::bind((CONFIG.get().unwrap().host, CONFIG.get().unwrap().port)) {
        Ok(listener) => listener,
        Err(error) => panic!("Problem binding TcpListener: {error:?}"),
    };

    let acceptor = match TlsAcceptor::new(CONFIG.get().unwrap().identity.clone()) {
        Ok(acceptor) => acceptor,
        Err(error) => panic!("Problem reading identity file: {error:?}"),
    };
    let acceptor = Arc::new(acceptor);


    let pool = ThreadPool::new(4);

    let mut method_handlers = HttpMethodHandlerTable::new();
    method_handlers.use_defaults();

    for stream in listener.incoming() {
        let _stream = match stream {
            Ok(_stream) => _stream,
            Err(error) => panic!("Error receiving packet from listener: {error:?}"),
        };

        let thread_method_handlers = method_handlers.clone();

        let acceptor = acceptor.clone();

        pool.execute(move || {
            let stream = acceptor.accept(_stream).unwrap();
            handle_connection(stream, &thread_method_handlers);
        });
    }
}
