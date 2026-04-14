use std::{
    fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = match TcpListener::bind("127.0.0.1:7878") {
        Ok(listener) => listener,
        Err(error) => panic!("Problem binding TcpListener: {error:?}"),
    };

    for stream in listener.incoming() {
        let _stream = match stream {
            Ok(_stream) => _stream,
            Err(error) => panic!("Error receiving packet from listener: {error:?}"),
        };

        println!("Connection Established.");

        handle_connection(_stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let status_line = "HTTP/1.1 200 OK/r/n/r/n";
    let contents = match fs::read_to_string("hello.html") {
        Ok(contents) => contents,
        Err(error) => panic!("Error reading file {error:?}"),
    };
    let length = contents.len();

    let response = 
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");


    match stream.write_all(response.as_bytes()) {
        Ok(result) => result,
        Err(error) => panic!("Could not write response: {error:?}"),
    };
}