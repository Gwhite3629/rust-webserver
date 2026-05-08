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

use colored::Colorize;


fn __internal_process<'req>(req: HttpRequest, state: &mut NonceTracker) -> HttpResponse {

    let mut currentstatus: HttpStatus;
    let mut headers = HttpFields::new();

    let mut file_contents = Vec::<u8>::new();
    let mut contents = Vec::<u8>::new();

    let mut req_path = Path::new(req.target.path.as_str());
    let base = Path::new(&CONFIG.get().unwrap().servers.get(&req.server_name).unwrap().path);
    let path = Path::new(base);
    let mut final_path: String;

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
            // Remove nonce after 100 requests
            let t = state.map.get(&a.name);
            if t.is_some() {
                if t.unwrap().n >= 100 {
                    state.map.remove(&a.name);
                    dval = None;
                }
            }
            // Check if realm matches request and send new 401 if it doesn't
            match dval {
                Some(d) => {
                    match dfun {
                        Some(func) => {
                            let realm = a.name.clone();
                            let nonce = match state.get(&a.name) {
                                Some(n) => Some(n.val.clone()),
                                None => None,
                            };
                            let userauth = UserAuth {
                                user: a.user,
                                pass: a.pass,
                                realm,
                                nonce: nonce.clone(),
                            };
                            let userdata = AuthData {
                                method: req.method,
                                uri: req.target,
                                nonce,
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
                                                UserAuthResult::CHANGEREALM => {
                                                    currentstatus = HttpStatus::Unauthorized;
                                                    match a.method {
                                                        AuthType::BASIC => {
                                                            headers.insert("WWW-authenticate", format!("{} realm=\"{}\"", AuthType::as_str(&a.method), a.name).as_str());
                                                        },
                                                        AuthType::DIGEST => {
                                                            println!("{}","Sending 401".green());
                                                            headers.insert("WWW-authenticate", 
                                                            format!("{} realm=\"{}\",qop=\"auth\",nonce=\"{}\",",
                                                            AuthType::as_str(&a.method),
                                                            a.name,
                                                            match state.get(&a.name) {
                                                                Some(a) => a.val.clone(),
                                                                None => {
                                                                    state.insert(a.name.clone());
                                                                    state.get(&a.name).unwrap().val.clone()
                                                                },
                                                            },) .as_str());
                                                        },
                                                    };
                                                    f = None;
                                                },
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
                            println!("{}","Sending 401".green());
                            headers.insert("WWW-authenticate", 
                            format!("{} realm=\"{}\",qop=\"auth\",nonce=\"{}\",",
                            AuthType::as_str(&a.method),
                            a.name,
                            match state.get(&a.name) {
                                Some(a) => a.val.clone(),
                                None => {
                                    state.insert(a.name.clone());
                                    state.get(&a.name).unwrap().val.clone()
                                },
                            },) .as_str());
                        },
                    };
                    f = None;
                },
            }
        },
        None => (),
    }

    if currentstatus == HttpStatus::Forbidden {
        final_path = path.join("403.html").to_str().unwrap().to_string();
        f = Some(File::open(&final_path));
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
        println!("{currentstatus:#?}");
        if (currentstatus != HttpStatus::Unauthorized) & (currentstatus != HttpStatus::Forbidden) {
            currentstatus = HttpStatus::BadRequest;
        }
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