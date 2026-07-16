use std::io::BufRead;
use std::net::SocketAddr;
use std::sync::{
    Arc,
    Mutex,
};

use crate::{CONFIG, HttpRequest, HttpResponse, ServerState, URI};

#[derive(Debug, Clone)]
pub struct Proxy {
    pub dest: SocketAddr,
    pub sources: Option<Vec<URI>>,
}

pub struct ProxyResponse {

}

pub struct ProxyProcessor {}

impl ProxyProcessor {
    pub fn new() -> Self {
        todo!();
    }

    pub fn handle_connection(
        buf: &[u8],
        name: String,
        state: Arc<Mutex<ServerState>>,
    ) -> Option<ProxyResponse> {
        if buf.is_empty() {
            return None;
        }

        let raw_request: Vec<String> = buf
            .lines()
            .map(|result| match result {
                Ok(r) => r,
                Err(_) => String::new(),
            })
            .take_while(|line| !line.is_empty())
            .collect();

        if raw_request.is_empty() {
            return None;
        }
    
        let request: HttpRequest = HttpRequest::new(raw_request, name.clone());

        let protocol = "HTTPS";
        let host = request.headers.get("Host").unwrap();
        let target = request.target.to_string();
        
        let uristring = protocol.to_string() + "://" + host + target.as_str();

        println!("{uristring:#?}");

        let proxies= CONFIG
            .get()
            .unwrap()
            .servers
            .get(&request.server_name)
            .unwrap()
            .proxy.clone();

        let mut dest: Option<SocketAddr> = None;

        for pr in proxies.unwrap() {
            match pr.sources {
                Some(l) => {
                    for s in l {
                        if s.to_string().starts_with(uristring.as_str()) {
                            dest = Some(pr.dest);
                            break;
                        }
                    }
                },
                None => dest = Some(pr.dest),
            }
            if dest.is_some() {
                break;
            }
        }

        // Proxy has been received and route is understood
        // Send raw request to local server, await response and process
        // Forward response to original sender as it was received.
        
        todo!()
    }

}