use std::io::BufRead;

use crate::{CONFIG, HttpRequest, HttpResponse, ServerState};

//const MB: usize = 1000000;
const KB: usize = 1000;

pub struct HttpProcessor {}

impl HttpProcessor {
    pub fn new() -> Self {
        todo!();
    }

    pub fn handle_connection(
        buf: &[u8],
        name: String,
        state: &mut ServerState,
    ) -> Option<HttpResponse> {
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

        let response: HttpResponse = match CONFIG
            .get()
            .unwrap()
            .servers
            .get(&name)
            .unwrap()
            .method_handlers
            .get(request.method)
        {
            Some(call) => call(request, state),
            None => return None,
        };

        Some(response)
    }

    pub fn to_chunks(res: HttpResponse) -> impl Iterator<Item = Vec<u8>> {
        let mut chunks = Vec::new();

        for chunk in res.content.chunks(1000 * KB) {
            let mut wr = Vec::<u8>::new();
            wr.append(
                &mut format!("{:x}", chunk.len())
                    .to_ascii_lowercase()
                    .as_bytes()
                    .to_vec(),
            );
            wr.append(&mut "\r\n".as_bytes().to_vec());
            wr.append(&mut chunk.to_vec());
            wr.append(&mut "\r\n".as_bytes().to_vec());
            chunks.push(wr);
        }

        let mut wr = Vec::<u8>::new();
        wr.append(&mut "0\r\n".as_bytes().to_vec());
        wr.append(&mut "\r\n".as_bytes().to_vec());
        chunks.push(wr);

        chunks.into_iter()
    }
}
