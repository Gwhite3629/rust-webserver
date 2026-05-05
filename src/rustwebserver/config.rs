use std::fs::{File};
use std::io::{BufReader, Read};
use std::net::{IpAddr, SocketAddr};
use std::collections::HashMap;
use std::sync::OnceLock;
use std::path::PathBuf;

use crate::{HttpFieldHandlerTable, HttpMethodHandlerTable, Processor};

pub static CONFIG: OnceLock<GlobalConfig> = OnceLock::new();

#[derive(Clone, PartialEq)]
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

// This is a pretty generic config that could be used for more protocols
pub struct HttpConfig {
    pub protocol: Protocol,
    pub path: String,
    pub addr: SocketAddr,
    pub method_handlers: HttpMethodHandlerTable,
    pub field_handlers: HttpFieldHandlerTable,
    //pub root_certs: PathBuf,
    pub certs: PathBuf,
    //pub crls: Vec<PathBuf>,
    pub privkey: PathBuf,
}

pub struct GlobalConfig {
    pub servers: HashMap<String, HttpConfig>,
}

impl HttpConfig {
    pub fn new(args: Vec<String>) -> Self {
        let params = HttpConfig::parse(args);
        HttpConfig { 
            protocol: Protocol::from_str(params.get("Protocol").unwrap().as_str()).unwrap(),
            path: params.get("Path").unwrap().to_string(), 
            addr: SocketAddr::new(
                params.get("Host").unwrap().parse::<IpAddr>().unwrap(),
                params.get("Port").unwrap().parse::<u16>().unwrap()
            ),
            method_handlers: HttpConfig::populate_methods(),
            field_handlers: HttpConfig::populate_fields(),
            certs: PathBuf::from(params.get("Cert").unwrap().to_string()),
            privkey: PathBuf::from(params.get("Key").unwrap().to_string()),
        }
    }

    pub fn parse(args: Vec<String>) -> HashMap<String,String> {
        let mut params: HashMap<String, String> = HashMap::new();

        let mut pairs = Vec::<(&str, &str)>::new();

        for () in args.iter().map(|l| 
            match l.trim().split_once(":") {
            Some(p) => pairs.push(p),
            None => (),
        }){}

        for (key, val) in pairs {
            params.insert(key.trim().to_string(), val.trim().to_string());
        }
        params
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

        let servers: Vec<&str> = contents.split("}\n").collect();

        for server in servers {
            let sp = server.split("{\n").collect::<Vec<&str>>();
            let name: String = sp[0].to_string();
            let params: Vec<String> = sp[1].split("\n").take_while(|line| !line.is_empty()).map(|s| s.to_string()).collect();

            confs.insert(name, HttpConfig::new(params));
        }

        confs
    }
}