use std::{
    fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use rustwebserver::ThreadPool;

use rustwebserver::HttpRequest;

fn main() {
    let listener = match TcpListener::bind("127.0.0.1:7878") {
        Ok(listener) => listener,
        Err(error) => panic!("Problem binding TcpListener: {error:?}"),
    };
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let _stream = match stream {
            Ok(_stream) => _stream,
            Err(error) => panic!("Error receiving packet from listener: {error:?}"),
        };

        pool.execute(|| {
            handle_connection(_stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf_reader = BufReader::new(&stream);

    let http_request: Vec<_> = buf_reader
        .by_ref()
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    //let request_line = buf_reader.lines().next().unwrap().unwrap();

    println!("Request: {http_request:#?}");

    if !http_request.is_empty() {

        let request: HttpRequest = HttpRequest::new(http_request.clone());

        println!("Processed Request:\n{request}");

        let len = match request.headers.get("Content-length") {
            Some(res) => res.parse::<usize>().unwrap(),
            None => 0,
        };

        let mut content: Vec<u8> = vec![0; len];

        buf_reader
            .read_exact(&mut content).expect("read failed");

        let http_string: String = String::from_utf8(content).unwrap();

        println!("Content: {:#?}", http_string);

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

        match stream.write_all(response.as_bytes()) {
            Ok(result) => result,
            Err(error) => panic!("Could not write response: {error:?}"),
        };
    }
}