use std::{
    collections::HashMap,
    io::{
        self,
        Error,
        ErrorKind,
        prelude::*
    },
    mem,
    net::{
        Shutdown,
        SocketAddr,
    },
    path::Path,
    sync::{
        Arc,
        Mutex
    },
    os::unix::io::AsRawFd,
};

use rustls::{
    ServerConfig, ServerConnection,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
};

use mio::{
    Interest, Poll, Token, event::Event, net::{TcpListener, TcpStream}
};

use libc;

use colored::Colorize;

use crate::{CONFIG, FileCache, HttpProcessor, NonceTracker, config::Protocol};

pub const LISTENER: mio::Token = mio::Token(0);

//pub const HTTP2: &str = "PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";


#[derive(Clone)]
pub enum Processor {
    HTTP,
}

pub struct ServerState {
    pub noncehandler: NonceTracker,
    pub file_cache: FileCache,
}

pub struct Server {
    name: String,
    listener: Arc<Mutex<TcpListener>>,
    protocol: Protocol,
    connections: Arc<Mutex<HashMap<mio::Token, Arc<Mutex<OpenConnection>>>>>,
    next_id: Arc<Mutex<usize>>,
    tls_config: Option<Arc<ServerConfig>>,
    engine: Processor,
    state: Arc<Mutex<ServerState>>,
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
    pub fn new(name: String, listener: Arc<Mutex<TcpListener>>) -> Self {
        let mode = CONFIG
            .get()
            .unwrap()
            .servers
            .get(&name)
            .unwrap()
            .protocol
            .clone();

        let tls_config = match mode {
            Protocol::HTTP => None,
            Protocol::HTTPS => Some(tls_setup(name.clone())),
        };

        let addr = listener.lock().unwrap().local_addr().unwrap();
        println!("{} is watching: {}", name, addr);

        Server {
            name,
            listener,
            protocol: mode.clone(),
            connections: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(2)),
            tls_config,
            engine: mode.to_processor(),
            state: Arc::new(Mutex::new(ServerState { noncehandler: NonceTracker::new(), file_cache: FileCache::new() })),
        }
    }

    pub fn accept_new_connection(self: Arc<Self>, reg: Arc<Mutex<Poll>>) -> Result<(), Error> {
        println!("Entered accept function");
        loop {
            // Hastily grab socket and unlock
            let socket_res: Result<(TcpStream, SocketAddr), Error>;
            match self.listener.lock() {
                Ok(l) => {
                    socket_res = l.accept();
                },
                Err(_) => panic!("Can't acquire listener"),
            }

            match socket_res {
                Ok((socket, _)) => {
                    println!("Accepting new connection on {}", self.name);
                    let tls_conn = match self.protocol {
                        Protocol::HTTP => None,
                        Protocol::HTTPS => Some(
                            ServerConnection::new(self.tls_config.as_ref().unwrap().clone())
                                .unwrap(),
                        ),
                    };

                    unsafe {
                        let optval: libc::c_int = 1;
                        let ret = libc::setsockopt(
                            socket.as_raw_fd(),
                            libc::SOL_SOCKET,
                            libc::SO_REUSEPORT,
                            &optval as *const _ as *const libc::c_void,
                            mem::size_of_val(&optval) as libc::socklen_t,
                        );
                        if ret != 0 {
                            return Err(io::Error::last_os_error());
                        }
                    }

                    let token: Token;

                    match self.next_id.lock() {
                        Ok(mut i) => {
                            token = Token(*i);
                            *i = *i + 1;
                        },
                        Err(_) => panic!("Couldn't acquire lock"),
                    }

                    let mut connection = OpenConnection::new(
                        self.name.clone(),
                        socket,
                        token,
                        tls_conn,
                        self.engine.clone(),
                    );
                    connection.register(reg.clone());
                    match self.connections.lock() {
                        Ok(mut c) => {
                            c.insert(token, Arc::new(Mutex::new(connection)));
                        },
                        Err(_) => panic!("Couldn't acquire lock"),
                    };
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(()),
                Err(error) => {
                    println!("Error encountered while accepting connection: {error:?}");
                    return Err(error);
                }
            }
        }
    }
    pub fn established_connection(self: Arc<Self>, reg: Arc<Mutex<Poll>>, event: &Event) {
        let token = event.token();
        match self.connections.lock() {
            Ok(mut c) => {
                if c.contains_key(&token) {
                    println!("Got established connection on {}", self.name);

                    let mut cull_flag = false;

                    let state_clone = Arc::clone(&self.state);
                    match c.get(&token).unwrap().lock() {
                        Ok(mut c) => {
                            c.ready(reg, event, state_clone);
                            if c.is_closed() {
                                cull_flag = true;
                            }
                        },
                        Err(_) => (),
                    };
                    if cull_flag {
                        c.remove(&token);
                    }
                }
            },
            Err(_) => panic!("Couldn't acquire lock"),
        }
    }
}

impl OpenConnection {
    pub fn new(
        name: String,
        socket: TcpStream,
        token: Token,
        tls_conn: Option<ServerConnection>,
        serv: Processor,
    ) -> Self {
        let protocol = CONFIG
            .get()
            .unwrap()
            .servers
            .get(&name)
            .unwrap()
            .protocol
            .clone();
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

    fn ready(&mut self, reg: Arc<Mutex<Poll>>, event: &Event, state: Arc<Mutex<ServerState>>) {
        if event.is_readable() {
            //println!("Reading event");
            match self.protocol {
                Protocol::HTTP => {
                    self.try_text_read(state);
                }
                Protocol::HTTPS => {
                    self.tls_read();
                    self.try_text_read_tls(state);
                    if self.protocol == Protocol::HTTPS {
                        println!("ALPN Protocol: {}", format!("{:#?}", String::from_utf8(self.tls_conn.as_ref().unwrap().alpn_protocol().unwrap().to_ascii_lowercase())).yellow());
                        println!("TLS Protocol: {}", format!("{:#?}", self.tls_conn.as_ref().unwrap().protocol_version()).yellow());
                        println!("Handshake type: {}", format!("{:#?}", self.tls_conn.as_ref().unwrap().handshake_kind()).yellow());
                    }
                }
            }
        }

        if event.is_writable() {
            //println!("Writing event");
            match self.protocol {
                Protocol::HTTP => {}
                Protocol::HTTPS => {
                    self.tls_write();
                }
            }
        }

        if self.closing {
            //println!("Closing event");
            let _ = self.socket.shutdown(Shutdown::Both);
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
                    //println!("Would block");
                    return;
                }

                // Log stuff
                println!("Read error: {error:?}");
                self.closing = true;
                return;
            }
            Ok(0) => {
                //println!("Closing");
                self.closing = true;
                return;
            }
            Ok(_) => {
                //println!("TLS read successful");
            }
        };

        if let Err(error) = self.tls_conn.as_mut().unwrap().process_new_packets() {
            // Log stuff
            println!("Cannot process packet: {error:?}");

            self.tls_write();
            self.closing = true;
        }
    }

    fn try_text_read_tls(&mut self, state: Arc<Mutex<ServerState>>) {
        if let Ok(io_state) = self.tls_conn.as_mut().unwrap().process_new_packets() {
            //println!("got io_state");
            if let Some(mut early_data) = self.tls_conn.as_mut().unwrap().early_data() {
                let mut buf = Vec::new();
                early_data.read_to_end(&mut buf).unwrap();
                //println!("Got early text");

                if !buf.is_empty() {
                    //println!("Processing early text");
                    self.incoming_text(&buf, state);
                    return;
                }
            }

            //let n = io_state.plaintext_bytes_to_read();
            //println!("bytes: {n}");

            if io_state.plaintext_bytes_to_read() > 0 {
                //println!("Processing plain test");
                let mut buf = vec![0u8; io_state.plaintext_bytes_to_read()];

                self.tls_conn
                    .as_mut()
                    .unwrap()
                    .reader()
                    .read_exact(&mut buf)
                    .unwrap();

                self.incoming_text(&buf, state);
            }
        }
    }

    fn try_text_read(&mut self, state: Arc<Mutex<ServerState>>) {
        let mut buf = vec![0u8; 4 * 1024];
        let n: usize;
        match self.socket.read(&mut buf) {
            Err(error) => {
                if let ErrorKind::WouldBlock = error.kind() {
                    //println!("Would block");
                    return;
                }

                // Log stuff
                println!("Read error: {error:?}");
                self.closing = true;
                return;
            }
            Ok(0) => {
                //println!("Closing");
                self.closing = true;
                return;
            }
            Ok(n_read) => {
                n = n_read;
                //println!("RAW read successful");
            }
        };

        //println!("bytes: {n}");
        if n > 0 {
            self.incoming_text(&buf, state);
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

    fn incoming_text(&mut self, buf: &[u8], state: Arc<Mutex<ServerState>>) {
        match self.engine {
            Processor::HTTP => match self.protocol {
                Protocol::HTTP => {
                    let res = match HttpProcessor::handle_connection(
                        buf,
                        self.name.clone(),
                        state,
                    ) {
                        Some(res) => res,
                        None => return,
                    };
                    if !self.sent_http_response {
                        self.socket.write(res.to_string().as_bytes()).unwrap();
                        self.sent_http_response = true;
                    }
                    for chunk in HttpProcessor::to_chunks(res) {
                        match self.socket.write_all(&chunk) {
                            Ok(_) => (),
                            Err(error) => {
                                if let ErrorKind::BrokenPipe = error.kind() {
                                    continue;
                                }

                                // Log stuff
                                println!("Write error: {error:?}");
                                self.closing = true;
                                return;
                            }
                        }
                    }
                    self.closing = true;
                    return;
                }
                Protocol::HTTPS => {
                    let res = match HttpProcessor::handle_connection(
                        buf,
                        self.name.clone(),
                        state,
                    ) {
                        Some(res) => res,
                        None => {
                            self.tls_conn.as_mut().unwrap().send_close_notify();
                            return;
                        }
                    };
                    if !self.sent_http_response {
                        self.tls_conn
                            .as_mut()
                            .unwrap()
                            .writer()
                            .write_all(res.to_string().as_bytes())
                            .unwrap();
                        self.sent_http_response = true;
                    }
                    for chunk in HttpProcessor::to_chunks(res) {
                        self.tls_conn
                            .as_mut()
                            .unwrap()
                            .writer()
                            .write(&chunk)
                            .unwrap();
                    }
                    self.tls_conn.as_mut().unwrap().send_close_notify();
                }
            }
        };
    }

    fn register(&mut self, poll: Arc<Mutex<Poll>>) {
        let event_set = self.event_set();
        match poll.lock() {
            Ok(p) => {
                p.registry().register(&mut self.socket, self.token, event_set)
                .unwrap();
            },
            Err(_) => (),
        };
    }

    fn reregister(&mut self, poll: Arc<Mutex<Poll>>) {
        let event_set = self.event_set();
        match poll.lock() {
            Ok(p) => {
                p.registry().reregister(&mut self.socket, self.token, event_set)
                .unwrap();
            },
            Err(_) => (),
        };
    }

    fn deregister(&mut self, poll: Arc<Mutex<Poll>>) {
        match poll.lock() {
            Ok(p) => {
                p.registry().deregister(&mut self.socket).unwrap();
            },
            Err(_) => (),
        };
    }

    fn event_set(&self) -> Interest {
        let (rd, wr) = match self.protocol {
            Protocol::HTTP => {
                // Need a better metric here
                (true, false)
            }
            Protocol::HTTPS => (
                self.tls_conn.as_ref().unwrap().wants_read(),
                self.tls_conn.as_ref().unwrap().wants_write(),
            ),
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
    let certs = load_certs(
        &CONFIG
            .get()
            .unwrap()
            .servers
            .get(&name)
            .unwrap()
            .certs
            .as_ref()
            .unwrap(),
    );

    let privkey = load_private_key(
        &CONFIG
            .get()
            .unwrap()
            .servers
            .get(&name)
            .unwrap()
            .privkey
            .as_ref()
            .unwrap(),
    );

    let mut config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, privkey)
        .expect("bad certificates or private key");

    config.alpn_protocols = vec![b"http/1.1".to_vec(), b"http/1.0".to_vec()];

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
