use std::{
    collections::HashMap, 
    io::{BufReader, prelude::*}, 
    net::TcpStream, 
    path::Path, 
    sync::Arc
};

use rustls::{RootCertStore, ServerConfig, ServerConnection};
use rustls::pki_types::{CertificateDer, pem::PemObject, PrivateKeyDer, CertificateRevocationListDer};
use rustls::server::WebPkiClientVerifier;

use mio::net::TcpListener;

use crate::{
    HttpRequest,
    HttpResponse,
    HttpMethodHandlerTable,
    CONFIG
};

enum ServerError {

}

struct Server {
    listener: TcpListener,
    connections: HashMap<mio::Token, OpenConnection>,
    next_id: usize,
    tls_config: Arc<ServerConfig>,
}

struct OpenConnection {
    socket: TcpStream,
    token: mio::Token,
    closing: bool,
    closed: bool,
    tls_conn: ServerConnection,
    back: Option<TcpStream>,
    sent_http_response: bool,
}

//const MB: usize = 1000000;
const KB: usize = 1000;

impl Server {
    fn new(listener: TcpListener, tls_config: Arc<ServerConfig>) -> Self {
        Server { 
            listener, 
            connections: HashMap::new(), 
            next_id: 2,
            tls_config
        }
    }

    //fn accept_new_connection(&mut self, )
}

// certs
// crl list
// server private key

fn tls_setup() -> Arc<ServerConfig> {

    let root_certs = load_certs(&CONFIG.get().unwrap().root_certs);
    let mut auth_roots = RootCertStore::empty();
    for root in root_certs {
        auth_roots.add(root).unwrap();
    }
    let crl_list = load_crls(CONFIG.get().unwrap().crls.iter());

    let auth = 
    WebPkiClientVerifier::builder(auth_roots.into())
            .with_crls(crl_list)
            .build()
            .unwrap();

    let certs = load_certs(&CONFIG.get().unwrap().certs);
    let privkey = load_private_key(&CONFIG.get().unwrap().privkey);


    let config = ServerConfig::builder()
        .with_client_cert_verifier(auth)
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

fn load_crls(
    filenames: impl Iterator<Item = impl AsRef<Path>>,
) -> Vec<CertificateRevocationListDer<'static>> {
    filenames
        .map(|filename| {
            CertificateRevocationListDer::from_pem_file(filename).expect("cannot read CRL file")
        })
        .collect()
}


pub fn handle_connection(mut stream: &TcpStream, mut _conn: &ServerConnection, method_handlers: &HttpMethodHandlerTable) {
    let mut buf_reader = BufReader::new(&mut stream);

    let raw_request: Vec<_> = buf_reader
        .by_ref()
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();


    // Terminate connection if no request made
    if raw_request.is_empty() {
        return;
    }

    let request: HttpRequest = HttpRequest::new(raw_request);

    let response: HttpResponse = match method_handlers.get(request.method) {
        Some(call) => call(request),
        None => return,
    };

    // Write response header
    match stream.write_all(response.to_string().as_bytes()) {
        Ok(result) => result,
        Err(error) => panic!("Could not write response headers: {error:?}"),
    };

    //send_chunked(stream, response);

}

/*
fn send_chunked(mut stream: TlsStream<TcpStream>, response: HttpResponse) {

    // Write chunked response
    for chunk in response.content.chunks(1*KB) {
        let mut wr = Vec::<u8>::new();
        wr.append(&mut format!("{:x}", chunk.len()).to_ascii_lowercase().as_bytes().to_vec());
        wr.append(&mut "\r\n".as_bytes().to_vec());
        wr.append(&mut chunk.to_vec());
        wr.append(&mut "\r\n".as_bytes().to_vec());
        match stream.write(&wr) {
            Ok(_) => (),
            Err(error) => panic!("Could not write response headers: {error:?}"),
        }
    }
    // Indicate chunked finished
    let mut wr = Vec::<u8>::new();
    wr.append(&mut "0\r\n".as_bytes().to_vec());
    wr.append(&mut "\r\n".as_bytes().to_vec());
    match stream.write(&wr) {
        Ok(_) => (),
        Err(error) => panic!("Could not write response headers: {error:?}"),
    }
}
*/