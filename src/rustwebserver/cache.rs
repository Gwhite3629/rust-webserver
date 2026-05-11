use std::fs::File;
use std::{collections::HashMap};

use std::io::{BufReader, Read};

use colored::Colorize;

pub enum CacheType {
    IDENTITY,
    GZIP,
}

pub enum CacheTry<T> {
    GOTCORRECT(T),
    GOTPLAIN(T),
    FAIL,
}

// Cache will currently just read or remove, no write-back or dirty bit
pub struct FileCache {
    plaintext: HashMap<String, Vec<u8>>,
    gzipped: HashMap<String, Vec<u8>>,
}

impl FileCache {
    pub fn new() -> Self {
        FileCache{
            plaintext: HashMap::new(),
            gzipped: HashMap::new(),
        }
    }

    fn read_to_cache(&mut self, fname: &String) {
        let mut content = Vec::<u8>::new();
        let f = Some(File::open(fname));
        let mut buf_reader = BufReader::new(f.unwrap().ok().unwrap());
    
        buf_reader.read_to_end(&mut content).unwrap();

        self.plaintext.insert(fname.clone(), content);
    }

    fn get(&self, fname: &String, ctype: CacheType) -> Option<&Vec<u8>> {
        match ctype {
            CacheType::GZIP => {
                self.gzipped.get(fname)
            },
            CacheType::IDENTITY => {
                self.plaintext.get(fname)
            }
        }
    }

    fn contains_key(&self, fname: &String, ctype: CacheType) -> bool {
        match ctype {
            CacheType::GZIP => {
                self.gzipped.contains_key(fname)
            },
            CacheType::IDENTITY => {
                self.plaintext.contains_key(fname)
            }
        }
    }

    pub fn cache(&mut self, fname: String, ctype: &Option<CacheType>, val: Vec<u8>) {
        match ctype {
            Some(c) => {
                match c {
                    CacheType::GZIP => {
                        self.gzipped.insert(fname, val);
                    },
                    CacheType::IDENTITY => {
                        self.plaintext.insert(fname, val);
                    },
                }
            }
            None => (),
        }
    }

    pub fn try_get(&mut self, fname: &String, ctype: &Option<CacheType>) -> CacheTry<&Vec<u8>> {
        match ctype {
            Some(c) => {
                match c {
                    CacheType::GZIP => {
                        if self.contains_key(fname, CacheType::GZIP) {
                            println!("{}", "Cache Hit.".green());
                            CacheTry::GOTCORRECT(self.get(fname, CacheType::GZIP).unwrap())
                        } else {
                            println!("{}", "Cache Miss.".red());
                            self.read_to_cache(fname);
                            CacheTry::GOTPLAIN(self.get(fname, CacheType::IDENTITY).unwrap())
                        }
                    },
                    CacheType::IDENTITY => {
                        if self.contains_key(fname, CacheType::IDENTITY) {
                            println!("{}", "Cache Hit.".green());
                            CacheTry::GOTPLAIN(self.get(fname, CacheType::IDENTITY).unwrap())
                        } else {
                            println!("{}", "Cache Miss.".red());
                            self.read_to_cache(fname);
                            CacheTry::GOTPLAIN(self.get(fname, CacheType::IDENTITY).unwrap())
                        }
                    },
                }
            },
            None => CacheTry::FAIL,
        }
    }
}