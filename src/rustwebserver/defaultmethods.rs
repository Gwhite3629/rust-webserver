
use std::path::Path;

use crate::cache::{CacheTry, CacheType};
use crate::{HttpResponse, ServerState};
use crate::file::{is_valid_path, resolve_path};
use crate::handler::{AuthData, UserAuth, UserAuthResult};
use crate::{
    AuthType, DefaultFields, HttpFields, HttpRequest, HttpStatus, WriterType,
};

use crate::HttpFieldHandler;

use crate::RequestEffect;
use crate::RequestState;
use crate::file::get_mimetype;

use crate::config::CONFIG;

use colored::Colorize;

fn __internal_process<'req>(req: HttpRequest, state: &mut ServerState) -> HttpResponse {
    let mut currentstatus: HttpStatus;
    let mut headers = HttpFields::new();

    let mut contents = Vec::<u8>::new();

    let mut req_path = Path::new(req.target.path.as_str());
    let base = Path::new(
        &CONFIG
            .get()
            .unwrap()
            .servers
            .get(&req.server_name)
            .unwrap()
            .path,
    );
    let path = Path::new(base);
    let mut final_path: String;

    println!("Unresolved path: {req_path:#?}");

    let (req_pathbuf, auth) = resolve_path(&req_path, &req.server_name);
    req_path = req_pathbuf.as_path();

    println!("Resolved path: {req_path:#?}");

    if is_valid_path(&req_path, &req.server_name) {
        currentstatus = HttpStatus::OK;
        final_path = path
            .join(&req_path.strip_prefix("/").unwrap())
            .to_str()
            .unwrap()
            .to_string();
    } else {
        currentstatus = HttpStatus::NotFound;
        final_path = path.join("404.html").to_str().unwrap().to_string();
    }

    // File to be sent to client
    let mut read_file: bool = true;

    // Captured values used for compression writer dispatch
    let mut wfun: Option<Box<HttpFieldHandler>> = None;
    let mut wval: Option<String> = None;

    let mut dfun: Option<Box<HttpFieldHandler>> = None;
    let mut dval: Option<String> = None;

    // Loop over request headers and call custom methods
    for (key, val) in req.headers {
        let () = match CONFIG
            .get()
            .unwrap()
            .servers
            .get(&req.server_name)
            .unwrap()
            .field_handlers
            .get(&key)
        {
            Some(fun) => {
                match DefaultFields::from_string(key).unwrap() {
                    DefaultFields::ACCEPT => {
                        println!("Got accept header.");
                        //fun(val, &mut RequestState{path: &final_path});
                        ()
                    }
                    DefaultFields::ACCEPTENCODING => {
                        println!("Parsing encoding:");
                        wfun = Some(Box::new(fun));
                        wval = Some(val);
                        ()
                    }
                    DefaultFields::AUTHORIZATION => {
                        println!("Parsing authorization:");
                        dfun = Some(Box::new(fun));
                        dval = Some(val);
                        ()
                    }
                    DefaultFields::CONNECTION => {
                        println!("Got connection header.");
                        ()
                    }
                    _ => (),
                }
            }
            None => (),
        };
    }

    match auth {
        Some(a) => {
            // Remove nonce after 100 requests
            let t = state.noncehandler.map.get(&a.name);
            if t.is_some() {
                if t.unwrap().n >= 100 {
                    state.noncehandler.map.remove(&a.name);
                    dval = None;
                }
            }
            // Check if realm matches request and send new 401 if it doesn't
            match dval {
                Some(d) => match dfun {
                    Some(func) => {
                        let realm = a.name.clone();
                        let nonce = match state.noncehandler.get(&a.name) {
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
                        match func(d, &mut RequestState { auth: &userdata }) {
                            RequestEffect::DECODER(dec) => match dec {
                                Some(mut call) => match call(&userauth) {
                                    UserAuthResult::AUTHORIZED => {
                                        currentstatus = HttpStatus::OK;
                                    }
                                    UserAuthResult::UNAUTHORIZED => {
                                        currentstatus = HttpStatus::Forbidden;
                                        read_file = false;
                                    }
                                    UserAuthResult::CHANGEREALM => {
                                        currentstatus = HttpStatus::Unauthorized;
                                        match a.method {
                                            AuthType::BASIC => {
                                                headers.insert(
                                                    "WWW-authenticate",
                                                    format!(
                                                        "{} realm=\"{}\"",
                                                        AuthType::as_str(&a.method),
                                                        a.name
                                                    )
                                                    .as_str(),
                                                );
                                            }
                                            AuthType::DIGEST => {
                                                println!("{}", "Sending 401".green());
                                                headers.insert("WWW-authenticate",
                                                            format!("{} realm=\"{}\",qop=\"auth\",nonce=\"{}\",",
                                                            AuthType::as_str(&a.method),
                                                            a.name,
                                                            match state.noncehandler.get(&a.name) {
                                                                Some(a) => a.val.clone(),
                                                                None => {
                                                                    state.noncehandler.insert(a.name.clone());
                                                                    state.noncehandler.get(&a.name).unwrap().val.clone()
                                                                },
                                                            },) .as_str());
                                            }
                                        };
                                        read_file = false;
                                    }
                                },
                                None => (),
                            },
                            RequestEffect::WRITER(_) => (),
                        }
                    }
                    None => {
                        currentstatus = HttpStatus::InternalServerError;
                        read_file = false;
                    }
                },
                None => {
                    currentstatus = HttpStatus::Unauthorized;
                    match a.method {
                        AuthType::BASIC => {
                            headers.insert(
                                "WWW-authenticate",
                                format!("{} realm=\"{}\"", AuthType::as_str(&a.method), a.name)
                                    .as_str(),
                            );
                        }
                        AuthType::DIGEST => {
                            println!("{}", "Sending 401".green());
                            headers.insert(
                                "WWW-authenticate",
                                format!(
                                    "{} realm=\"{}\",qop=\"auth\",nonce=\"{}\",",
                                    AuthType::as_str(&a.method),
                                    a.name,
                                    match state.noncehandler.get(&a.name) {
                                        Some(a) => a.val.clone(),
                                        None => {
                                            state.noncehandler.insert(a.name.clone());
                                            state.noncehandler.get(&a.name).unwrap().val.clone()
                                        }
                                    },
                                )
                                .as_str(),
                            );
                        }
                    };
                    read_file = false;
                }
            }
        }
        None => (),
    }

    let f: Option<&Vec<u8>>;

    if currentstatus == HttpStatus::Forbidden {
        final_path = path.join("403.html").to_str().unwrap().to_string();
        read_file = true;
    }

    let cache_type: Option<CacheType>;
    if wval.clone().is_some_and(|t| t.contains("gzip")) {
        headers.insert("content-encoding", "gzip");
        cache_type = Some(CacheType::GZIP);
    } else {
        headers.insert("content-encoding", "identity");
        cache_type = Some(CacheType::IDENTITY);
    };

    let mut cache_write: bool = false;

    // Read file and write contents to buffer
    if read_file {
        match state.file_cache.try_get(&final_path, &cache_type) {
            CacheTry::GOTCORRECT(cont) => {
                f = Some(&cont);
                contents.extend_from_slice(f.unwrap());

                headers.insert("content-length", f.unwrap().len().to_string().as_str());
                headers.insert("content-type", get_mimetype(&final_path).as_str());
                headers.insert("transfer-encoding", "chunked");

            }
            CacheTry::GOTPLAIN(cont) => {
                f = Some(&cont);

                let mut contents_container = RequestState {
                    contents: &mut contents,
                };

                let writer: WriterType;
                writer = match wfun {
                    Some(wfun) => match wval {
                        Some(val) => {
                            match wfun(val, &mut contents_container) {
                                RequestEffect::WRITER(w) => w,
                                RequestEffect::DECODER(_) => None,
                            }
                        }
                        None => None,
                    },
                    None => None,
                };

                //let mut gzip_writer = GzEncoder::new(&mut contents, Compression::default());
                if writer.is_some() {
                    match writer.unwrap()(f.unwrap()) {
                        Ok(result) => {
                            headers.insert("content-length", result.to_string().as_str());
                            headers.insert("content-type", get_mimetype(&final_path).as_str());
                            headers.insert("transfer-encoding", "chunked");
                        }
                        Err(error) => panic!("Could not write response content: {error:?}"),
                    }
                } else {
                    currentstatus = HttpStatus::InternalServerError;
                }
                cache_write = true;
            }
            CacheTry::FAIL => {
                println!("{currentstatus:#?}");
                if (currentstatus != HttpStatus::Unauthorized) & (currentstatus != HttpStatus::Forbidden) {
                    currentstatus = HttpStatus::BadRequest;
                }
            },
        }
    } else {
        println!("{currentstatus:#?}");
        if (currentstatus != HttpStatus::Unauthorized) & (currentstatus != HttpStatus::Forbidden) {
            currentstatus = HttpStatus::BadRequest;
        }
    }

    if cache_write {
        state.file_cache.cache(final_path.clone(), cache_type, contents.clone());
    }

    HttpResponse {
        version: req.version.clone(),
        status: currentstatus,
        headers: headers,
        content: contents,
    }
}

pub fn handle_get<'req>(req: HttpRequest, state: &mut ServerState) -> HttpResponse {
    let res = __internal_process(req, state);

    HttpResponse {
        version: res.version,
        status: res.status,
        headers: res.headers,
        content: res.content,
    }
}

pub fn handle_head(req: HttpRequest, state: &mut ServerState) -> HttpResponse {
    let res = __internal_process(req, state);

    HttpResponse {
        version: res.version,
        status: res.status,
        headers: res.headers,
        content: Vec::new(),
    }
}

pub fn handle_options(_req: HttpRequest, _state: &mut ServerState) -> HttpResponse {
    HttpResponse::new()
}

pub fn handle_trace(req: HttpRequest, _state: &mut ServerState) -> HttpResponse {
    let mut currentstatus = HttpStatus::OK;
    let mut headers = HttpFields::new();

    let file_contents = Vec::<u8>::from(req.to_string());
    let mut contents = Vec::<u8>::new();

    // Captured values used for compression writer dispatch
    let mut wfun: Option<Box<HttpFieldHandler>> = None;
    let mut wval: Option<String> = None;

    // Loop over request headers and call custom methods
    for (key, val) in req.headers {
        let () = match CONFIG
            .get()
            .unwrap()
            .servers
            .get(&req.server_name)
            .unwrap()
            .field_handlers
            .get(&key)
        {
            Some(fun) => {
                match DefaultFields::from_string(key).unwrap() {
                    DefaultFields::ACCEPT => {
                        println!("Got accept header.");
                        //fun(val, &mut RequestState{path: &final_path});
                        ()
                    }
                    DefaultFields::ACCEPTENCODING => {
                        println!("Parsing encoding:");
                        wfun = Some(Box::new(fun));
                        wval = Some(val);
                        ()
                    }
                    DefaultFields::CONNECTION => {
                        println!("Got connection header.");
                        ()
                    }
                    _ => (),
                }
            }
            None => (),
        };
    }

    {
        let mut contents_container = RequestState {
            contents: &mut contents,
        };

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
                }
                None => None,
            },
            None => None,
        };

        if writer.is_some() {
            match writer.unwrap()(&file_contents) {
                Ok(result) => {
                    headers.insert("content-length", result.to_string().as_str());
                    headers.insert("transfer-encoding", "chunked");
                }
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
