use std::fs::File;
use std::path::Path;
use std::io::{BufReader, Read};

use crate::file::is_valid_path;
use crate::{HttpRequest, HttpStatus, HttpFields};
use crate::HttpResponse;

use crate::config::CONFIG;

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

pub fn handle_get(req: HttpRequest) -> HttpResponse {

    let mut currentstatus: HttpStatus;

    let mut contents = Vec::<u8>::new();

    let mut flag = true;

    let req_path = req.target.path.as_str();

    if !is_valid_path(&Path::new(req_path)) {
        currentstatus = HttpStatus::BadRequest;
    } else {

        let mut f = File::open(CONFIG.get().unwrap().path.clone() + req_path);

        if f.as_ref().is_ok() {
            currentstatus = HttpStatus::OK;
        } else {
            currentstatus = HttpStatus::NotFound;
            f = File::open(CONFIG.get().unwrap().path.clone() + "404.html");
            if !f.as_ref().is_ok() {
                flag = false;
            }
        };

        if flag {
            let mut buf_reader: BufReader<File> = BufReader::new(f.ok().unwrap());
            match buf_reader.read_to_end(&mut contents) {
                Ok(_) => (),
                Err(_) => currentstatus = HttpStatus::InternalServerError,
            }
        } else {
            currentstatus = HttpStatus::BadRequest;
        }
    }

    HttpResponse {
        version: req.version.clone(),
        status: currentstatus,
        headers: HttpFields::new(),
        content: contents,
    }
}

pub fn handle_head(_req: HttpRequest) -> HttpResponse {
    HttpResponse::new()
}

pub fn handle_options(_req: HttpRequest) -> HttpResponse {
    HttpResponse::new()
}

pub fn handle_trace(_req: HttpRequest) -> HttpResponse {
    HttpResponse::new()
}