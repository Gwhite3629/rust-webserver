use std::{
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
};

use rustwebserver::{
    HttpConfig,
    HttpRequest,
    HttpResponse,
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

    let listener = match TcpListener::bind((CONFIG.get().unwrap().host, CONFIG.get().unwrap().port)) {
        Ok(listener) => listener,
        Err(error) => panic!("Problem binding TcpListener: {error:?}"),
    };
    let pool = ThreadPool::new(4);

    let mut method_handlers = HttpMethodHandlerTable::new();
    method_handlers.use_defaults();

    for stream in listener.incoming() {
        let _stream = match stream {
            Ok(_stream) => _stream,
            Err(error) => panic!("Error receiving packet from listener: {error:?}"),
        };

        let thread_method_handlers = method_handlers.clone();

        pool.execute(move || {
            handle_connection(_stream, &thread_method_handlers);
        });
    }
}

fn handle_connection(mut stream: TcpStream, method_handlers: &HttpMethodHandlerTable) {
    let mut buf_reader = BufReader::new(&stream);

    let http_request: Vec<_> = buf_reader
        .by_ref()
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    //let request_line = buf_reader.lines().next().unwrap().unwrap();

    //println!("Request: {http_request:#?}");

    if http_request.is_empty() {
        return;
    }

    let request: HttpRequest = HttpRequest::new(http_request.clone());

    //println!("Processed Request:\n{request}");

    let len = match request.headers.get("Content-length") {
        Some(res) => res.parse::<usize>().unwrap(),
        None => 0,
    };

    let mut content: Vec<u8> = vec![0; len];

    buf_reader
        .read_exact(&mut content).expect("read failed");

    //let http_string: String = String::from_utf8(content).unwrap();

    //println!("Content: {:#?}", http_string);

    let response: HttpResponse = method_handlers.get(request.method).unwrap()(request);
/*
    let (status_line, filename) = match http_request[0].as_str() {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = match fs::read_to_string(filename) {
        Ok(contents) => contents,
        Err(error) => panic!("Error reading file: {error:?}"),
    };
    let length = contents.len();

    let response = format!(
        "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
    );
*/
    match stream.write_all(response.to_string().as_bytes()) {
        Ok(result) => result,
        Err(error) => panic!("Could not write response headers: {error:?}"),
    };

    for chunk in response.content.chunks(100) {
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

    let mut wr = Vec::<u8>::new();
    wr.append(&mut "0\r\n".as_bytes().to_vec());
    wr.append(&mut "\r\n".as_bytes().to_vec());
    match stream.write(&wr) {
        Ok(_) => (),
        Err(error) => panic!("Could not write response headers: {error:?}"),
    }

}