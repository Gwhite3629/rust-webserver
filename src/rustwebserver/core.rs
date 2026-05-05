use std::{
    collections::HashMap,
    io::{
        BufReader, Error, ErrorKind, prelude::*
    },
    net::Shutdown,
    path::Path,
    sync::Arc
};

use rustls::{
    //RootCertStore,
    ServerConfig,
    ServerConnection,
    pki_types::{
        CertificateDer,
        pem::PemObject,
        PrivateKeyDer,
        //CertificateRevocationListDer
    },
    //server::WebPkiClientVerifier,
};

use mio::{
    Interest,
    Registry,
    Token,
    event::Event,
    Events,
    Poll,
    net::{
        TcpListener,
        TcpStream
    },
};

use crate::{
    CONFIG, HttpProcessor, config::Protocol
};

pub const LISTENER: mio::Token = mio::Token(0);

#[derive(Clone)]
pub enum Processor {
    HTTP,
}

pub struct Server {
    name: String,
    protocol: Protocol,
    listener: TcpListener,
    connections: HashMap<mio::Token, OpenConnection>,
    next_id: usize,
    tls_config: Option<Arc<ServerConfig>>,
    engine: Processor,
}

struct OpenConnection {
    name: String,
    protocol: Protocol,
    socket: TcpStream,
    token: mio::Token,
    closing: bool,
    closed: bool,
    tls_conn: Option<ServerConnection>,
    sent_http_response: bool,
    engine: Processor,
}

impl Server {
    pub fn new(name: String) -> Self {

        let mode = CONFIG.get().unwrap().servers.get(&name).unwrap().protocol.clone();

        let tls_config = match mode {
            Protocol::HTTP => {
                None
            },
            Protocol::HTTPS => {
                Some(tls_setup(name.clone()))
            }
        };

        let listener = match TcpListener::bind(CONFIG.get().unwrap().servers.get(&name).unwrap().addr) {
            Ok(listener) => listener,
            Err(error) => panic!("Problem binding TcpListener: {error:?}"),
        };
        let addr = listener.local_addr().unwrap();
        println!("{} is watching: {}", name, addr);

        Server { 
            name,
            protocol: mode.clone(),
            listener, 
            connections: HashMap::new(), 
            next_id: 2,
            tls_config,
            engine: mode.to_processor()
        }
    }

    pub fn start(&mut self) {

        let mut poll = match Poll::new() {
            Ok(p) => p,
            Err(error) => panic!("Failed to create os-poll structure {error:?}"),
        };
        let mut events = Events::with_capacity(256);

        match poll.registry().register(&mut self.listener, LISTENER, Interest::READABLE) {
            Ok(_) => (),
            Err(error) => panic!("Failed to register listener: {error:?}"),
        };

        loop {
            match poll.poll(&mut events, None) {
                Ok(_) => {},
                Err(error) if error.kind() == ErrorKind::Interrupted => continue ,
                Err(error) => {
                    panic!("Poll failed: {error:?}");
                },
            }

            for event in events.iter() {
                println!("Got event");
                match event.token() {
                    LISTENER => self.accept_new_connection(poll.registry()).expect("Error accepting connection."),
                    _ => self.established_connection(poll.registry(), event),
                }
            }
        }
    }

    pub fn accept_new_connection(&mut self, reg: &Registry) -> Result<(), Error> {
        loop {
            match self.listener.accept() {
                Ok((socket, _)) => {
                    println!("Accepting new connection on {}", self.name);
                    let tls_conn = match self.protocol {
                        Protocol::HTTP => {
                            None
                        },
                        Protocol::HTTPS => {
                            Some(ServerConnection::new(self.tls_config.as_ref().unwrap().clone()).unwrap())
                        },
                    };

                    let token = Token(self.next_id);
                    self.next_id += 1;

                    let mut connection = OpenConnection::new(self.name.clone(), socket, token, tls_conn, self.engine.clone());
                    connection.register(reg);
                    self.connections.insert(token, connection);

                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(()),
                Err(error) => {
                    println!("Error encountered while accepting connection: {error:?}");
                    return Err(error);
                }
            }
        }
    }
    pub fn established_connection(&mut self, reg: &Registry, event: &Event) {
        let token = event.token();

        if self.connections.contains_key(&token) {
            println!("Got established connection on {}", self.name);
            self.connections.get_mut(&token).unwrap().ready(reg, event);

            if self.connections[&token].is_closed() {
                self.connections.remove(&token);
            }
        }
    }
}

impl OpenConnection {
    pub fn new(name: String, socket: TcpStream, token: Token, tls_conn: Option<ServerConnection>, serv: Processor) -> Self {
        let protocol = CONFIG.get().unwrap().servers.get(&name).unwrap().protocol.clone();
        Self {
            name,
            protocol,
            socket,
            token,
            closing: false,
            closed: false,
            tls_conn,
            sent_http_response: false,
            engine: serv,
        }
    }

    fn ready(&mut self, reg: &Registry, event: &Event) {
        if event.is_readable() {
            println!("Reading event");
            match self.protocol {
                Protocol::HTTP => {
                    self.try_text_read();
                },
                Protocol::HTTPS => {
                    self.tls_read();
                    self.try_text_read_tls();
                }
            }
        }

        if event.is_writable() {
            println!("Writing event");
            match self.protocol {
                Protocol::HTTP => {
                    
                },
                Protocol::HTTPS => {
                    self.tls_write();
                }
            }
        }

        if self.closing {
            println!("Closing event");
            let _ = self
                .socket
                .shutdown(Shutdown::Both);
            self.closed = true;
            self.deregister(reg);
        } else {
            self.reregister(reg);
        }
    }

    fn tls_read(&mut self) {
        match self.tls_conn.as_mut().unwrap().read_tls(&mut self.socket) {
            Err(error) => {
                if let ErrorKind::WouldBlock = error.kind() {
                    println!("Would block");
                    return;
                }
                
                // Log stuff
                println!("Read error: {error:?}");
                self.closing = true;
                return;
            }
            Ok(0) => {
                println!("Closing");
                self.closing = true;
                return;
            }
            Ok(_) => {println!("TLS read successful");}
        };

        if let Err(error) = self.tls_conn.as_mut().unwrap().process_new_packets() {
            // Log stuff
            println!("Cannot process packet: {error:?}");

            self.tls_write();
            self.closing = true;
        }
    }

    fn try_text_read_tls(&mut self) {
        if let Ok(io_state) = self.tls_conn.as_mut().unwrap().process_new_packets() {
            println!("got io_state");
            if let Some(mut early_data) = self.tls_conn.as_mut().unwrap().early_data() {
                let mut buf = Vec::new();
                early_data.read_to_end(&mut buf).unwrap();
                println!("Got early text");

                if !buf.is_empty() {
                    println!("Processing early text");
                    self.incoming_text(&buf);
                    return;
                }
            }

            let n = io_state.plaintext_bytes_to_read();
            println!("bytes: {n}");

            if io_state.plaintext_bytes_to_read() > 0 {
                println!("Processing plain test");
                let mut buf = vec![0u8; io_state.plaintext_bytes_to_read()];

                self.tls_conn.as_mut().unwrap().reader().read_exact(&mut buf).unwrap();

                self.incoming_text(&buf);
            }
        }
    }


    fn try_text_read(&mut self) {
        let mut buf = vec![0u8; 4*1024];
        let mut n = 0;
        match self.socket.read(&mut buf) {
            Err(error) => {
                if let ErrorKind::WouldBlock = error.kind() {
                    println!("Would block");
                    return;
                }
                
                // Log stuff
                println!("Read error: {error:?}");
                self.closing = true;
                return;
            }
            Ok(0) => {
                println!("Closing");
                self.closing = true;
                return;
            }
            Ok(n_read) => {
                n = n_read;
                println!("RAW read successful");}
        };

        //let n = self.socket.read(&mut buf).unwrap();
        println!("bytes: {n}");
        if n > 0 {
            self.incoming_text(&buf);
        }
    }

    fn tls_write(&mut self) {
        let rc = self.tls_conn.as_mut().unwrap().write_tls(&mut self.socket);
        if rc.is_err() {
            // Log stuff
            println!("Write failed: {rc:?}");
            self.closing = true;
        }
    }

    fn incoming_text(&mut self, buf: &[u8]) {
        let print_str = String::from_utf8(buf.to_ascii_lowercase()).unwrap();
        match self.engine {
            Processor::HTTP => {
                match self.protocol {
                    Protocol::HTTP => {
                        let res = match HttpProcessor::handle_connection(buf, self.name.clone()) {
                            Some(res) => res,
                            None => return,
                        };
                        if !self.sent_http_response {
                            self.socket.write_all(res.to_string().as_bytes()).unwrap();
                            self.sent_http_response = true;
                        }
                        for chunk in HttpProcessor::to_chunks(res) {
                            self.socket.write(&chunk).unwrap();
                        }
                        self.closing = true;
                        return;
                    },
                    Protocol::HTTPS => {
                        let res = match HttpProcessor::handle_connection(buf, self.name.clone()) {
                            Some(res) => res,
                            None => {
                                self.tls_conn.as_mut().unwrap().send_close_notify();
                                return;
                            }
                        };
                        if !self.sent_http_response {
                            self.tls_conn.as_mut().unwrap().writer().write_all(res.to_string().as_bytes()).unwrap();
                            self.sent_http_response = true;
                        }
                        for chunk in HttpProcessor::to_chunks(res) {
                            self.tls_conn.as_mut().unwrap().writer().write(&chunk).unwrap();
                        }
                        self.tls_conn.as_mut().unwrap().send_close_notify();
                    },
                }
            }
        };
    }

    fn register(&mut self, reg: &Registry) {
        let event_set = self.event_set();
        reg.register(&mut self.socket, self.token, event_set).unwrap();
    }

    fn reregister(&mut self, reg: &mio::Registry) {
        let event_set = self.event_set();
        reg.reregister(&mut self.socket, self.token, event_set).unwrap();
    }

    fn deregister(&mut self, reg: &Registry) {
        reg.deregister(&mut self.socket).unwrap();
    }

    fn event_set(&self) -> Interest {
        let (rd, wr) = match self.protocol {
            Protocol::HTTP => {
                (true, true)
            },
            Protocol::HTTPS => {
                (self.tls_conn.as_ref().unwrap().wants_read(), self.tls_conn.as_ref().unwrap().wants_write())
            },
        };

        if rd && wr {
            Interest::READABLE | Interest::WRITABLE
        } else if wr {
            Interest::WRITABLE
        } else {
            Interest::READABLE
        }
    }

    fn is_closed(&self) -> bool {
        self.closed
    }
}



// certs
// crl list
// server private key

pub fn tls_setup(name: String) -> Arc<ServerConfig> {

    let certs = load_certs(&CONFIG.get().unwrap().servers.get(&name).unwrap().certs);
    let privkey = load_private_key(&CONFIG.get().unwrap().servers.get(&name).unwrap().privkey);

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            certs,
        privkey
        )
        .expect("bad certificates or private key");

    Arc::new(config)
}

fn load_certs(filename: &Path) -> Vec<CertificateDer<'static>> {
    CertificateDer::pem_file_iter(filename)
        .expect("cannot open certificate file")
        .map(|result| result.unwrap())
        .collect()
}

fn load_private_key(filename: &Path) -> PrivateKeyDer<'static> {
    PrivateKeyDer::from_pem_file(filename).expect("cannot read private key file")
}
