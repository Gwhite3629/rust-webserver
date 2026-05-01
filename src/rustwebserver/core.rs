use std::{
    collections::HashMap,
    io::{
        Error,
        ErrorKind,
        prelude::*
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
    net::{
        TcpListener,
        TcpStream
    },
};

use crate::{
    HttpProcessor,
    CONFIG
};

#[derive(Clone)]
pub enum Processor {
    HTTP,
}

pub struct Server {
    listener: TcpListener,
    connections: HashMap<mio::Token, OpenConnection>,
    next_id: usize,
    tls_config: Arc<ServerConfig>,
    engine: Processor,
}

struct OpenConnection {
    socket: TcpStream,
    token: mio::Token,
    closing: bool,
    closed: bool,
    tls_conn: ServerConnection,
    sent_http_response: bool,
    engine: Processor,
}

impl Server {
    pub fn new(listener: TcpListener, tls_config: Arc<ServerConfig>, mode: Processor) -> Self {
        Server { 
            listener, 
            connections: HashMap::new(), 
            next_id: 2,
            tls_config,
            engine: match mode {
                Processor::HTTP => Processor::HTTP
            }
        }
    }

    pub fn accept_new_connection(&mut self, reg: &Registry) -> Result<(), Error> {
        loop {
            match self.listener.accept() {
                Ok((socket, _)) => {
                    println!("Accepting new connection");
                    let tls_conn = ServerConnection::new(self.tls_config.clone()).unwrap();

                    let token = Token(self.next_id);
                    self.next_id += 1;

                    let mut connection = OpenConnection::new(socket, token, tls_conn, self.engine.clone());
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
            println!("Got established connection");
            self.connections.get_mut(&token).unwrap().ready(reg, event);

            if self.connections[&token].is_closed() {
                self.connections.remove(&token);
            }
        }
    }
}

impl OpenConnection {
    pub fn new(socket: TcpStream, token: Token, tls_conn: ServerConnection, serv: Processor) -> Self {
        Self {
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
            self.tls_read();
            self.try_text_read();
        }

        if event.is_writable() {
            println!("Writing event");
            self.tls_write();
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
        match self.tls_conn.read_tls(&mut self.socket) {
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

        if let Err(error) = self.tls_conn.process_new_packets() {
            // Log stuff
            println!("Cannot process packet: {error:?}");

            self.tls_write();
            self.closing = true;
        }
    }

    fn try_text_read(&mut self) {
        if let Ok(io_state) = self.tls_conn.process_new_packets() {
            println!("got io_state");
            if let Some(mut early_data) = self.tls_conn.early_data() {
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

                self.tls_conn.reader().read_exact(&mut buf).unwrap();

                self.incoming_text(&buf);
            }
        }
    }

    fn tls_write(&mut self) {
        let rc = self.tls_conn.write_tls(&mut self.socket);
        if rc.is_err() {
            // Log stuff
            println!("Write failed: {rc:?}");
            self.closing = true;
        }
    }

    fn incoming_text(&mut self, buf: &[u8]) {
        let print_str = String::from_utf8(buf.to_ascii_lowercase()).unwrap();
        println!("\tRAW TEXT: {print_str}");
        let () = match self.engine {
            Processor::HTTP => {
                let res = match HttpProcessor::handle_connection(buf) {
                    Some(res) => res,
                    None => {
                        self.tls_conn.send_close_notify();
                        return;
                    }
                };
                if !self.sent_http_response {
                    self.tls_conn.writer().write_all(res.to_string().as_bytes()).unwrap();
                    self.sent_http_response = true;
                }
                for chunk in HttpProcessor::to_chunks(res) {
                    self.tls_conn.writer().write(&chunk).unwrap();
                }
                self.tls_conn.send_close_notify();
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
        let rd = self.tls_conn.wants_read();
        let wr = self.tls_conn.wants_write();

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

pub fn tls_setup() -> Arc<ServerConfig> {
    /*
    let root_certs = load_certs(&CONFIG.get().unwrap().root_certs);
    let mut auth_roots = RootCertStore::empty();
    for root in root_certs {
        auth_roots.add(root).unwrap();
    }
    //let crl_list = load_crls(CONFIG.get().unwrap().crls.iter());

    let auth = 
    WebPkiClientVerifier::builder(auth_roots.into())
            .with_crls(crl_list)
            .build()
            .unwrap();
    */

    let certs = load_certs(&CONFIG.get().unwrap().certs);
    let privkey = load_private_key(&CONFIG.get().unwrap().privkey);

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

/*
fn load_crls(
    filenames: impl Iterator<Item = impl AsRef<Path>>,
) -> Vec<CertificateRevocationListDer<'static>> {
    filenames
        .map(|filename| {
            CertificateRevocationListDer::from_pem_file(filename).expect("cannot read CRL file")
        })
        .collect()
}
        */
