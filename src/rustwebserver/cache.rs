use std::fs::File;
use std::{collections::HashMap};

use std::io::{BufReader, Read};

use colored::Colorize;

#[derive(PartialEq, Clone)]
pub enum CacheType {
    IDENTITY,
    GZIP,
}

pub enum CacheTry<T> {
    GOTCORRECT(T),
    GOTPLAIN(T),
    FAIL,
}

pub struct Entry<T> {
    val: T,
    _has_write: bool,
    ctype: CacheType,
}

// Cache will currently just read or remove, no write-back or dirty bit
pub struct FileCache {
    map: HashMap<String, Entry<Vec<u8>>>,
    _size: usize,
}

impl FileCache {
    pub fn new() -> Self {
        FileCache{
            map: HashMap::new(),
            _size: 0,
        }
    }

    fn read_direct(&mut self, fname: &String) -> Option<Vec<u8>> {
        let mut content = Vec::<u8>::new();
        let f = Some(File::open(fname));
        let mut buf_reader = BufReader::new(f.unwrap().ok().unwrap());
    
        match buf_reader.read_to_end(&mut content) {
            Ok(_) => Some(content),
            Err(_) => None,
        }

    }

    fn get(&self, fname: &String) -> Option<&Vec<u8>> {
        match self.map.get(fname) {
            Some(e) => Some(&e.val),
            None => None,
        }
    }

    fn contains_key(&self, fname: &String, ctype: CacheType) -> bool {
        self.map.contains_key(fname) & self.map.get(fname).is_some_and(|e| e.ctype == ctype)
    }

    pub fn cache(&mut self, fname: String, ctype: Option<CacheType>, val: Vec<u8>) {
         self.map.insert(fname,
            Entry {
                val: val,
                _has_write: false,
                ctype: ctype.unwrap(),
            }
        );
    }
    pub fn read_to_cache(&mut self, fname: String, ctype: Option<CacheType>) {
        let ret = self.read_direct(&fname).unwrap();
        self.cache(fname, ctype,
            ret
        );
    }

    pub fn try_get(&mut self, fname: &String, ctype: &Option<CacheType>) -> CacheTry<Vec<u8>> {
        match ctype {
            Some(c) => {
                match c {
                    CacheType::GZIP => {
                        if self.contains_key(fname, CacheType::GZIP) {
                            println!("{}", "Cache Hit.".green());
                            CacheTry::GOTCORRECT(self.get(fname).unwrap().clone())
                        } else {
                            println!("{}", "Cache Miss.".red());
                            CacheTry::GOTPLAIN(self.read_direct(fname).unwrap())   
                        }
                    },
                    CacheType::IDENTITY => {
                        if self.contains_key(fname, CacheType::IDENTITY) {
                            println!("{}", "Cache Hit.".green());
                            CacheTry::GOTCORRECT(self.get(fname).unwrap().clone())
                        } else {
                            println!("{}", "Cache Miss.".red());
                            self.read_to_cache(fname.clone(), ctype.clone());
                            CacheTry::GOTCORRECT(self.get(fname).unwrap().clone())
                        }
                    },
                }
            },
            None => CacheTry::FAIL,
        }
    }
}