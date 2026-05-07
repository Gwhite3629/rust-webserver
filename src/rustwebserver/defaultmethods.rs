use std::fs::File;
use std::path::Path;
use std::io::{BufReader, Read};

use crate::file::{is_valid_path, resolve_path};
use crate::handler::{UserAuth, UserAuthResult, AuthData};
use crate::{AuthType, DefaultFields, HttpFields, HttpRequest, HttpStatus, NonceTracker, WriterType};
use crate::HttpResponse;

use crate::HttpFieldHandler;

use crate::file::get_mimetype;
use crate::RequestState;
use crate::RequestEffect;

use crate::config::CONFIG;


fn __internal_process<'req>(req: HttpRequest, state: &mut NonceTracker) -> HttpResponse {

    let mut currentstatus: HttpStatus;
    let mut headers = HttpFields::new();

    let mut file_contents = Vec::<u8>::new();
    let mut contents = Vec::<u8>::new();

    let mut req_path = Path::new(req.target.path.as_str());
    let base = Path::new(&CONFIG.get().unwrap().servers.get(&req.server_name).unwrap().path);
    let path = Path::new(base);
    let final_path: String;

    println!("Unresolved path: {req_path:#?}");

    let (req_pathbuf, auth) = resolve_path(&req_path, &req.server_name);
    req_path = req_pathbuf.as_path();

    println!("Resolved path: {req_path:#?}");

    if is_valid_path(&req_path, &req.server_name) {
        currentstatus = HttpStatus::OK;
        final_path = path.join(&req_path.strip_prefix("/").unwrap()).to_str().unwrap().to_string();
    } else {
        currentstatus = HttpStatus::NotFound;
        final_path = path.join("404.html").to_str().unwrap().to_string();
    }

    // File to be sent to client
    let mut f = Some(File::open(&final_path));

    // Captured values used for compression writer dispatch
    let mut wfun: Option<Box<HttpFieldHandler>> = None;
    let mut wval: Option<String> = None;

    let mut dfun: Option<Box<HttpFieldHandler>> = None;
    let mut dval: Option<String> = None;

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
                    DefaultFields::AUTHORIZATION => {
                        println!("Parsing authorization:");
                        dfun = Some(Box::new(fun));
                        dval = Some(val);
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

    match auth {
        Some(a) => {
            match dval {
                Some(d) => {
                    match dfun {
                        Some(func) => {
                            let realm = a.name.clone();
                            let userauth = UserAuth {
                                user: a.user,
                                pass: a.pass,
                                realm,
                                nonce: match d.contains("Basic") {
                                    true => None,
                                    false => {
                                        state.get(&a.name)
                                    },
                                }
                            };
                            let userdata = AuthData {
                                method: req.method,
                                uri: req.target,
                                nonce: match d.contains("Basic") {
                                    true => None,
                                    false => {
                                        state.get(&a.name)
                                    },
                                }
                            };
                            match func(d, &mut RequestState{auth: &userdata}){
                                RequestEffect::DECODER(dec) => {
                                    match dec {
                                        Some(mut call) => {
                                            match call(&userauth) {
                                                UserAuthResult::AUTHORIZED => {
                                                    currentstatus = HttpStatus::OK;
                                                }
                                                UserAuthResult::UNAUTHORIZED => {
                                                    currentstatus = HttpStatus::Forbidden;
                                                    f = None;
                                                }
                                            }
                                        },
                                        None => (),
                                    }
                                },
                                RequestEffect::WRITER(_) => (),
                            }
                        }
                        None => {
                            currentstatus = HttpStatus::InternalServerError;
                            f = None;
                        },
                    }
                },
                None => {
                    currentstatus = HttpStatus::Unauthorized;
                    match a.method {
                        AuthType::BASIC => {
                            headers.insert("WWW-authenticate", format!("{} realm=\"{}\"", AuthType::as_str(&a.method), a.name).as_str());
                        },
                        AuthType::DIGEST => {
                            headers.insert("WWW-authenticate", 
                            format!("{} realm=\"{}\",qop=\"auth\",nonce=\"{}\",",
                            AuthType::as_str(&a.method),a.name,state.get(&a.name).unwrap().val).as_str());

                        },
                    };
                    f = None;
                },
            }
        },
        None => (),
    }

    // Read file and write contents to buffer
    if f.is_some() {
        let mut buf_reader: BufReader<File> = BufReader::new(f.unwrap().ok().unwrap());
        let mut contents_container = RequestState{contents: &mut contents};
        match buf_reader.read_to_end(&mut file_contents) {
            Ok(_) => {
                
                let writer: WriterType;
                writer = match wfun {
                    Some(wfun) => match wval {
                        Some(val) => {
                            if val.contains("gzip") {
                               headers.insert("content-encoding", "gzip");
                            } else {
                                headers.insert("content-encoding", "identity");
                            };
                            match wfun(val, &mut contents_container) {
                                RequestEffect::WRITER(w) => w,
                                RequestEffect::DECODER(_) => None,
                            }
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


pub fn handle_get<'req>(req: HttpRequest, state: &mut NonceTracker) -> HttpResponse {
    let res = __internal_process(req, state);

    HttpResponse {
        version: res.version,
        status: res.status,
        headers: res.headers,
        content: res.content
    }
}

pub fn handle_head(req: HttpRequest, state: &mut NonceTracker) -> HttpResponse {
    let res = __internal_process(req, state);

    HttpResponse {
        version: res.version,
        status: res.status,
        headers: res.headers,
        content: Vec::new()
    }
}

pub fn handle_options(_req: HttpRequest, _state: &mut NonceTracker) -> HttpResponse {
    HttpResponse::new()
}

pub fn handle_trace(req: HttpRequest, _state: &mut NonceTracker) -> HttpResponse {
        
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

        let writer: WriterType;
        writer = match wfun {
            Some(wfun) => match wval {
                Some(val) => {
                    if val.contains("gzip") {
                        headers.insert("content-encoding", "gzip");
                    } else {
                        headers.insert("content-encoding", "identity");
                    };
                    match wfun(val, &mut contents_container) {
                        RequestEffect::WRITER(w) => w,
                        RequestEffect::DECODER(_) => None,
                    }
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