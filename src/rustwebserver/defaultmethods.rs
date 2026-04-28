use std::fs::File;
use std::path::Path;
use std::io::{BufReader, Read, Write, BufWriter};

use flate2::write::GzEncoder;
use flate2::Compression;

use crate::file::is_valid_path;
use crate::{CaseInsensitiveString, DefaultFields, HttpFields, HttpRequest, HttpStatus};
use crate::HttpResponse;

use crate::RequestEffect;
use crate::RequestState;

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

pub fn handle_get<'req>(req: HttpRequest) -> HttpResponse {

    let mut currentstatus: HttpStatus;

    let mut file_contents = Vec::<u8>::new();

    let mut contents = Vec::<u8>::new();

    let req_path = Path::new(req.target.path.as_str());

    let base = Path::new(&CONFIG.get().unwrap().path);

    let path = Path::new(base);

    let final_path: String;

    let headers: HttpFields;

    if is_valid_path(&req_path) {
        currentstatus = HttpStatus::OK;
        final_path = path.join(&req_path.strip_prefix("/").unwrap()).to_str().unwrap().to_string();
    } else {
        currentstatus = HttpStatus::NotFound;
        final_path = path.join("404.html").to_str().unwrap().to_string();
    }

    let f = File::open(&final_path);

    let binding = BufWriter::new(Vec::new());

    let mut writer =  RequestEffect{writer: Box::new(&binding)};

    for (key, val) in req.headers {
        let () = match CONFIG.get().unwrap().field_handlers.get(&key) {
            Some(fun) => {
                match DefaultFields::from_string(key).unwrap() {
                    DefaultFields::ACCEPT => {
                        fun(val, RequestState{path: &final_path}).unwrap();
                        ()},
                    DefaultFields::ACCEPTENCODING => {
                        writer = fun(val, RequestState{contents: &mut contents}).unwrap();
                        ()},
                    DefaultFields::CONNECTION => (),
                    _ => (),
                }
            },
            None => (),
        };
    }

    if f.as_ref().is_ok() {
        let mut buf_reader: BufReader<File> = BufReader::new(f.ok().unwrap());
        match buf_reader.read_to_end(&mut file_contents) {
            Ok(_) => {

                //let mut gzip_writer = GzEncoder::new(&mut contents, Compression::default());

                match writer.writer.write(&file_contents) {
                    Ok(result) => headers = HttpResponse::generate_get_headers(final_path, result),
                    Err(error) => panic!("Could not write response content: {error:?}"),
                }

            },
            Err(_) => {
                headers = HttpFields::new();
                currentstatus = HttpStatus::InternalServerError},
        }
    } else {
        currentstatus = HttpStatus::BadRequest;
        headers = HttpFields::new();
    }

    HttpResponse {
        version: req.version.clone(),
        status: currentstatus,
        headers: headers,
        content: contents,
    }
}

pub fn handle_head(req: HttpRequest) -> HttpResponse {
    
    let mut currentstatus: HttpStatus;

    let mut file_contents = Vec::<u8>::new();

    let mut contents = Vec::<u8>::new();

    let req_path = Path::new(req.target.path.as_str());

    let base = Path::new(&CONFIG.get().unwrap().path);

    let path = Path::new(base);

    let final_path: String;

    let headers: HttpFields;

    if is_valid_path(&req_path) {
        currentstatus = HttpStatus::OK;
        final_path = path.join(&req_path.strip_prefix("/").unwrap()).to_str().unwrap().to_string();
    } else {
        currentstatus = HttpStatus::NotFound;
        final_path = path.join("404.html").to_str().unwrap().to_string();
    }
    
    let f = File::open(&final_path);

    if f.as_ref().is_ok() {
        let mut buf_reader: BufReader<File> = BufReader::new(f.ok().unwrap());
        match buf_reader.read_to_end(&mut file_contents) {
            Ok(_) => {

                let mut gzip_writer = GzEncoder::new(&mut contents, Compression::default());

                match gzip_writer.write(&file_contents) {
                    Ok(result) => headers = HttpResponse::generate_get_headers(final_path, result),
                    Err(error) => panic!("Could not write response content: {error:?}"),
                }
            },
            Err(_) => {
                headers = HttpFields::new();
                currentstatus = HttpStatus::InternalServerError},
        }
    } else {
        currentstatus = HttpStatus::BadRequest;
        headers = HttpFields::new();
    }

    HttpResponse {
        version: req.version.clone(),
        status: currentstatus,
        headers: headers,
        content: Vec::new(),
    }
}

pub fn handle_options(_req: HttpRequest) -> HttpResponse {
    HttpResponse::new()
}

pub fn handle_trace(req: HttpRequest) -> HttpResponse {
        
    let currentstatus: HttpStatus;

    let file_contents = Vec::<u8>::from(req.to_string());

    let mut contents = Vec::<u8>::new();

    let headers: HttpFields;

    {
        let mut gzip_writer = GzEncoder::new(&mut contents, Compression::default());

        match gzip_writer.write(&file_contents) {
            Ok(result) => {
                headers = HttpResponse::generate_trace_headers(result);
                currentstatus = HttpStatus::OK},
            Err(error) => panic!("Could not write response content: {error:?}"),
        }
    }

    HttpResponse {
        version: req.version.clone(),
        status: currentstatus,
        headers: headers,
        content: contents,
    }
}