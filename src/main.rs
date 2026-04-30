use mio::net::TcpListener;
use std::sync::Arc;

use rustls::{
    server::{
        Acceptor,
    },
};

use rustwebserver::{
    HttpConfig,
    HttpMethodHandlerTable,
    ThreadPool,
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

    let _listener = match TcpListener::bind(CONFIG.get().unwrap().addr) {
        Ok(listener) => listener,
        Err(error) => panic!("Problem binding TcpListener: {error:?}"),
    };

    let acceptor = Acceptor::default();
    let _acceptor = Arc::new(acceptor);


    let _pool = ThreadPool::new(4);

    let mut method_handlers = HttpMethodHandlerTable::new();
    method_handlers.use_defaults();

    /*
    for stream in listener.incoming() {
        let mut _stream = match stream {
            Ok(_stream) => _stream,
            Err(error) => panic!("Error receiving packet from listener: {error:?}"),
        };

        let thread_method_handlers = method_handlers.clone();
        let mut acceptor = acceptor.clone();

        let accepted = loop {
            match acceptor.accept() {
                Ok(Some(accepted)) => break accepted,
                Ok(None) => continue,
                Err((e, mut alert)) => {
                    alert.write_all(&mut _stream).unwrap();
                    panic!("error accepting connection: {e}");
                }
            }
        };


        let config = test_pki.server_config(&args.crl_path, accepted.client_hello());
        let mut conn = match accepted.into_connection(config) {
            Ok(conn) => conn,
            Err((e, mut alert)) => {
                alert.write_all(&mut _stream).unwrap();
                panic!("error completing accepting connection: {e}");
            }
        };
        pool.execute(move || {
            acceptor.read_tls(&mut _stream).unwrap();
            handle_connection(&mut _stream, &mut conn, &thread_method_handlers);
        });
    }
    */
}
