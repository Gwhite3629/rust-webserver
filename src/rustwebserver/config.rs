use std::fs::File;
use std::io::{BufReader, BufRead};
use std::net::IpAddr;
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::HttpFieldHandlerTable;

pub static CONFIG: OnceLock<HttpConfig> = OnceLock::new();


pub struct HttpConfig {
    pub path: String,
    pub port: u16,
    pub host: IpAddr,
    pub field_handlers: HttpFieldHandlerTable,
}

impl HttpConfig {
    pub fn new(args: Vec<String>) -> Self {
        let params = HttpConfig::parse(args);
        HttpConfig { 
            path: params.get("Path").unwrap().to_string(), 
            port: params.get("Port").unwrap().parse::<u16>().unwrap(), 
            host: params.get("Host").unwrap().parse::<IpAddr>().unwrap(),
            field_handlers: HttpConfig::populate_fields(),
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

