use std::fs::{DirEntry, File};
use std::io::{BufReader, BufRead, Read};
use std::net::{IpAddr, SocketAddr};
use std::collections::HashMap;
use std::sync::OnceLock;
use std::path::PathBuf;

use rustls::{

};

use crate::HttpFieldHandlerTable;

pub static CONFIG: OnceLock<HttpConfig> = OnceLock::new();


pub struct HttpConfig {
    pub path: String,
    pub addr: SocketAddr,
    pub field_handlers: HttpFieldHandlerTable,
    pub root_certs: PathBuf,
    pub certs: PathBuf,
    pub crls: Vec<PathBuf>,
    pub privkey: PathBuf,
}

impl HttpConfig {
    pub fn new(args: Vec<String>) -> Self {
        let params = HttpConfig::parse(args);
        HttpConfig { 
            path: params.get("Path").unwrap().to_string(), 
            addr: SocketAddr::new(
                params.get("Host").unwrap().parse::<IpAddr>().unwrap(),
                params.get("Port").unwrap().parse::<u16>().unwrap()
            ),
            field_handlers: HttpConfig::populate_fields(),
            root_certs: PathBuf::from(params.get("Root").unwrap().to_string()),
            certs: PathBuf::from(params.get("Cert").unwrap().to_string()),
            crls: {
                let mut v = Vec::<PathBuf>::new();
                for d in PathBuf::from(params.get("CRL").unwrap().to_string()).read_dir().unwrap() {
                    let () = match d {
                        Ok(d) => v.push(PathBuf::from(d.file_name())),
                        Err(_) => (),
                    };
                }
                v
                },
            privkey: PathBuf::from(params.get("Key").unwrap().to_string()),
        }
    }

    pub fn parse(args: Vec<String>) -> HashMap<String,String> {
        let mut params: HashMap<String, String> = HashMap::new();
        let f: File = match File::open(&args[1]) {
            Ok(f) => f,
            Err(error) => panic!("Config file path invalid: {error:?}"),
        };

        let buf_reader: BufReader<File> = BufReader::new(f);

        let contents: Vec<String> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

        for (key, val) in contents.iter().map(|l| l.split_once(":").unwrap()) {
            params.insert(key.to_string(), val.trim().to_string());
        }

        params
    }

    pub fn populate_fields() -> HttpFieldHandlerTable{
        let mut ret = HttpFieldHandlerTable::new();
        ret.use_defaults();
        ret
    }
}

