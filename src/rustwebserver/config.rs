use std::fs::{File};
use std::io::{BufReader, Read};
use std::net::{IpAddr, SocketAddr};
use std::iter::zip;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::path::PathBuf;
use regex::Regex;
use lazy_static::lazy_static;

use crate::{HttpFieldHandlerTable, HttpMethodHandlerTable, Processor};

pub static CONFIG: OnceLock<GlobalConfig> = OnceLock::new();

#[derive(Clone, PartialEq, Debug)]
pub enum Protocol {
    HTTP,
    HTTPS
}


impl Protocol {
    pub fn from_str(protocol: &str) -> Option<Protocol> {
        let protocol = protocol.to_uppercase();
        match protocol.as_str() {
            "HTTP" => Some(Protocol::HTTP),
            "HTTPS" => Some(Protocol::HTTPS),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Protocol::HTTP => "HTTP",
            Protocol::HTTPS => "HTTPS",
        }
    }

    pub fn to_processor(&self) -> Processor {
        match self {
            Protocol::HTTP => Processor::HTTP,
            Protocol::HTTPS => Processor::HTTP,
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum AuthType {
    BASIC,
    DIGEST,
}

impl AuthType {
    pub fn from_str(auth: &str) -> Option<AuthType> {
        let auth = auth.to_uppercase();
        match auth.as_str() {
            "BASIC" => Some(AuthType::BASIC),
            "DIGEST" => Some(AuthType::DIGEST),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            AuthType::BASIC => "Basic",
            AuthType::DIGEST => "Digest",
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Auth {
    pub method: AuthType,
    pub user: String,
    pub pass: String,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Redirect {
    pub req_path: PathBuf,
    pub redirect: Option<PathBuf>,
    pub auth: Option<Auth>,
}

#[derive(Debug)]
// This is a pretty generic config that could be used for more protocols
pub struct HttpConfig {
    pub protocol: Protocol,
    pub path: String,
    pub addr: SocketAddr,
    pub method_handlers: HttpMethodHandlerTable,
    pub field_handlers: HttpFieldHandlerTable,
    //pub root_certs: PathBuf,
    pub certs: Option<PathBuf>,
    //pub crls: Vec<PathBuf>,
    pub privkey: Option<PathBuf>,
    pub root_redirect: Option<Redirect>,
    pub redirects: Option<Vec::<Redirect>>,
}

#[derive(Debug)]
pub struct GlobalConfig {
    pub servers: HashMap<String, HttpConfig>,
}

impl HttpConfig {
    pub fn new(args: Vec<String>, raw_redirects: Option<Vec<&str>>) -> Self {
        let (params, potential_redirects)  = HttpConfig::parse(args, raw_redirects);
        let mut root_redirect: Option<Redirect> = None;
        let mut redirects: Option<Vec<Redirect>> = None;
        match potential_redirects {
            Some(red) => {
                redirects = Some(Vec::new());
                let root: Option<&Redirect> = red.iter().find(|&s| s.req_path == PathBuf::from("/"));
                if  root.is_some() {
                    root_redirect = Some(root.unwrap().clone());
                    for r in red {
                        if r.req_path != PathBuf::from("/") {
                            redirects.as_mut().expect("").push(r);
                        }
                    }
                }
            },
            None => (),
        }
        HttpConfig { 
            protocol: Protocol::from_str(params.get("Protocol").unwrap().as_str()).unwrap(),
            path: params.get("Path").unwrap().to_string(), 
            addr: SocketAddr::new(
                params.get("Host").unwrap().parse::<IpAddr>().unwrap(),
                params.get("Port").unwrap().parse::<u16>().unwrap()
            ),
            method_handlers: HttpConfig::populate_methods(),
            field_handlers: HttpConfig::populate_fields(),
            certs: match params.get("Cert") {
                Some(p) => Some(PathBuf::from(p.to_string())),
                None => None,
            },
            privkey: match params.get("Key") {
                Some(p) => Some(PathBuf::from(p.to_string())),
                None => None,
            },
            root_redirect,
            redirects
        }
    }

    pub fn parse(args: Vec<String>, raw_redirects: Option<Vec<&str>>) -> (HashMap<String,String>, Option<Vec<Redirect>>) {
        let mut params: HashMap<String, String> = HashMap::new();
        let mut redirects: Option<Vec::<Redirect>> = None;

        let mut pairs = Vec::<(&str, &str)>::new();

        for () in args.iter().map(|l| 
            match l.trim().split_once(":") {
            Some(p) => pairs.push(p),
            None => (),
        }){}

        match raw_redirects {
            Some(raw) => {
                redirects = Some(Vec::new());
                for r in raw {
                    let p: PathBuf;
                    let mut t: Option<PathBuf> = None;
                    let mut a: Option<AuthType> = None;
                    let mut user = String::new();
                    let mut pass = String::new();
                    let mut inside: Vec<&str> = r.trim().lines().collect();
                    p = PathBuf::from(inside[0].split_once("(").unwrap().0);
                    inside.remove(0);
                    inside.pop();
                    let pairs: Vec::<(&str, &str)> = inside.into_iter().map(|l| l.split_once(":").unwrap()).collect();
                    for (left, right) in pairs {
                        match left.to_uppercase().as_str() {
                            "REDIRECT" => t = Some(PathBuf::from(right)),
                            "AUTH" => a = Some(AuthType::from_str(right).unwrap()),
                            "USER" => user = right.to_string(),
                            "PASS" => pass = right.to_string(),
                            _ => (),
                        }
                    }

                    let red = Redirect { 
                        req_path: p, 
                        redirect: t, 
                        auth: match a {
                            Some(a) => Some(Auth {
                                method: a,
                                user,
                                pass,
                            }),
                            None => None,
                        },
                    };
                    redirects.as_mut().expect("Redirect should exist.").push(red);
                }
            },
            None => (),
        }

        for (key, val) in pairs {
            params.insert(key.trim().to_string(), val.trim().to_string());
        }
        (params, redirects)
    }

    pub fn populate_methods() -> HttpMethodHandlerTable {
        let mut ret = HttpMethodHandlerTable::new();
        ret.use_defaults();
        ret
    }

    pub fn populate_fields() -> HttpFieldHandlerTable {
        let mut ret = HttpFieldHandlerTable::new();
        ret.use_defaults();
        ret
    }
}

lazy_static! {
        static ref SERVER_REGEX: Regex = Regex::new(
            r"(?m)(?<server>^[^\}\{]+)\ \{(?<config>[^\}]*)\}$"
        ).unwrap();
    } 

impl GlobalConfig {
    pub fn new(args: Vec<String>) -> Self {
        GlobalConfig {servers: GlobalConfig::parse(args)}
    }

    fn parse(args: Vec<String>) -> HashMap<String, HttpConfig> {
        let mut confs = HashMap::<String, HttpConfig>::new();

        let f: File = match File::open(&args[1]) {
            Ok(f) => f,
            Err(error) => panic!("Config file path invalid: {error:?}"),
        };

        let mut buf_reader: BufReader<File> = BufReader::new(f);

        let mut contents = String::new();

        buf_reader.read_to_string(&mut contents).unwrap();

        let mut servers = Vec::<&str>::new();
        let mut configs = Vec::<&str>::new();
        let mut redirects = Vec::<Option<Vec<&str>>>::new();
        for (_, [s, c]) in SERVER_REGEX.captures_iter(&contents).map(|c| c.extract()) {
            servers.push(s);
            let mut a: Vec<&str> = c.split("Location:").collect();
            configs.push(a.first().unwrap());
            a.remove(0);
            match a.is_empty() {
                true => redirects.push(None),
                false => redirects.push(Some(a)),
            }
        }

        for ((server, config), redirect) in zip(zip(servers, configs), redirects) {
            let params = config.trim().split('\n').take_while(|line| !line.is_empty()).map(|s| s.trim().to_string()).collect();
            confs.insert(server.trim().to_string(), HttpConfig::new(params, redirect));
        }

        confs
    }
}