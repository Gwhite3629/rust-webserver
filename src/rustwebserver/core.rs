use std::{
    io::{BufReader, prelude::*},
    net::{TcpStream},
};

use native_tls::TlsStream;

use crate::{
    HttpRequest,
    HttpResponse,
    HttpMethodHandlerTable,
};

const MB: usize = 1000000;

pub fn handle_connection(mut stream: TlsStream<TcpStream>, method_handlers: &HttpMethodHandlerTable) {
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

    send_chunked(stream, response);

}

fn send_chunked(mut stream: TlsStream<TcpStream>, response: HttpResponse) {

    // Write chunked response
    for chunk in response.content.chunks(1*MB) {
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