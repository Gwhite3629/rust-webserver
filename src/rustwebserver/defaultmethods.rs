use std::fs::File;
use std::path::Path;
use std::io::{BufReader, Read};
use std::io;

use crate::file::is_valid_path;
use crate::{DefaultFields, HttpFields, HttpRequest, HttpStatus};
use crate::HttpResponse;

use crate::HttpFieldHandler;

use crate::file::get_mimetype;
use crate::RequestState;

use crate::config::CONFIG;


fn __internal_process<'req>(req: HttpRequest) -> HttpResponse {

    let mut currentstatus: HttpStatus;
    let mut headers = HttpFields::new();

    let mut file_contents = Vec::<u8>::new();
    let mut contents = Vec::<u8>::new();

    let req_path = Path::new(req.target.path.as_str());
    let base = Path::new(&CONFIG.get().unwrap().servers.get(&req.server_name).unwrap().path);
    let path = Path::new(base);
    let final_path: String;


    if is_valid_path(&req_path, &req.server_name) {
        currentstatus = HttpStatus::OK;
        final_path = path.join(&req_path.strip_prefix("/").unwrap()).to_str().unwrap().to_string();
    } else {
        currentstatus = HttpStatus::NotFound;
        final_path = path.join("404.html").to_str().unwrap().to_string();
    }

    // File to be sent to client
    let f = File::open(&final_path);

    // Captured values used for compression writer dispatch
    let mut wfun: Option<Box<HttpFieldHandler>> = None;
    let mut wval: Option<String> = None;

    // Loop over request headers and call custom methods
    for (key, val) in req.headers {
        let () = match CONFIG.get().unwrap().servers.get(&req.server_name).unwrap().field_handlers.get(&key) {
            Some(fun) => {
                match DefaultFields::from_string(key).unwrap() {
                    DefaultFields::ACCEPT => {
                        println!("Got accept header.");
                        //fun(val, &mut RequestState{path: &final_path});
                        ()},
                    DefaultFields::ACCEPTENCODING => {
                        println!("Parsing encoding:");
                        wfun = Some(Box::new(fun));
                        wval = Some(val);
                        ()},
                    DefaultFields::CONNECTION => {
                        println!("Got connection header.");
                        ()},
                    _ => (),
                }
            },
            None => (),
        };
    }

    // Read file and write contents to buffer
    if f.as_ref().is_ok() {
        let mut buf_reader: BufReader<File> = BufReader::new(f.ok().unwrap());
        let mut contents_container = RequestState{contents: &mut contents};
        match buf_reader.read_to_end(&mut file_contents) {
            Ok(_) => {
                
                let writer: Option<Box<dyn FnMut(&[u8]) -> io::Result<usize>>>;
                writer = match wfun {
                    Some(wfun) => match wval {
                        Some(wval) => {
                            if wval.contains("gzip") {
                               headers.insert("content-encoding", "gzip");
                            } else {
                                headers.insert("content-encoding", "identity");
                            };
                            Some(wfun(wval, &mut contents_container))
                        },
                        None => None
                    },
                    None => None
                };

                //let mut gzip_writer = GzEncoder::new(&mut contents, Compression::default());
                if writer.is_some() {
                    match writer.unwrap()(&file_contents) {
                        Ok(result) => {
                            headers.insert("content-length", result.to_string().as_str());
                            headers.insert("content-type", get_mimetype(final_path).as_str());
                            headers.insert("transfer-encoding", "chunked");
                        },
                        Err(error) => panic!("Could not write response content: {error:?}"),
                    }
                } else {
                    currentstatus = HttpStatus::InternalServerError;
                }

            },
            Err(_) => {
                currentstatus = HttpStatus::InternalServerError},
        }
    } else {
        currentstatus = HttpStatus::BadRequest;
    }

    HttpResponse {
        version: req.version.clone(),
        status: currentstatus,
        headers: headers,
        content: contents,
    }
}


pub fn handle_get<'req>(req: HttpRequest) -> HttpResponse {
    let res = __internal_process(req);

    HttpResponse {
        version: res.version,
        status: res.status,
        headers: res.headers,
        content: res.content
    }
}

pub fn handle_head(req: HttpRequest) -> HttpResponse {
    let res = __internal_process(req);

    HttpResponse {
        version: res.version,
        status: res.status,
        headers: res.headers,
        content: Vec::new()
    }
}

pub fn handle_options(_req: HttpRequest) -> HttpResponse {
    HttpResponse::new()
}

pub fn handle_trace(req: HttpRequest) -> HttpResponse {
        
    let mut currentstatus = HttpStatus::OK;
    let mut headers = HttpFields::new();

    let file_contents = Vec::<u8>::from(req.to_string());
    let mut contents = Vec::<u8>::new();


    // Captured values used for compression writer dispatch
    let mut wfun: Option<Box<HttpFieldHandler>> = None;
    let mut wval: Option<String> = None;

    // Loop over request headers and call custom methods
    for (key, val) in req.headers {
        let () = match CONFIG.get().unwrap().servers.get(&req.server_name).unwrap().field_handlers.get(&key) {
            Some(fun) => {
                match DefaultFields::from_string(key).unwrap() {
                    DefaultFields::ACCEPT => {
                        println!("Got accept header.");
                        //fun(val, &mut RequestState{path: &final_path});
                        ()},
                    DefaultFields::ACCEPTENCODING => {
                        println!("Parsing encoding:");
                        wfun = Some(Box::new(fun));
                        wval = Some(val);
                        ()},
                    DefaultFields::CONNECTION => {
                        println!("Got connection header.");
                        ()},
                    _ => (),
                }
            },
            None => (),
        };
    }

    {
        let mut contents_container = RequestState{contents: &mut contents};

        let writer: Option<Box<dyn FnMut(&[u8]) -> io::Result<usize>>>;
        writer = match wfun {
            Some(wfun) => match wval {
                Some(wval) => {
                    if wval.contains("gzip") {
                        headers.insert("content-encoding", "gzip");
                    } else {
                        headers.insert("content-encoding", "identity");
                    };
                    Some(wfun(wval, &mut contents_container))
                },
                None => None
            },
            None => None
        };

        if writer.is_some() {
            match writer.unwrap()(&file_contents) {
                Ok(result) => {
                    headers.insert("content-length", result.to_string().as_str());
                    headers.insert("transfer-encoding", "chunked");
                },
                Err(error) => panic!("Could not write response content: {error:?}"),
            }
        } else {
            currentstatus = HttpStatus::InternalServerError;
        }
    }

    HttpResponse {
        version: req.version.clone(),
        status: currentstatus,
        headers: headers,
        content: contents,
    }
}