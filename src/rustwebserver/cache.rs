use std::fs::File;
use std::{collections::HashMap};

use std::io::{BufReader, Read};

use colored::Colorize;

// Cache will currently just read or remove, no write-back or dirty bit
pub struct FileCache (HashMap<String, Vec<u8>>);

impl FileCache {
    pub fn new() -> Self {
        FileCache(HashMap::new())
    }

    fn read_to_cache(&mut self, fname: &String) {
        let mut content = Vec::<u8>::new();
        let f = Some(File::open(fname));
        let mut buf_reader = BufReader::new(f.unwrap().ok().unwrap());
    
        buf_reader.read_to_end(&mut content).unwrap();

        self.0.insert(fname.clone(), content);
    }

    fn get(&self, fname: &String) -> Option<&Vec<u8>> {
        self.0.get(fname)
    }

    fn contains_key(&self, fname: &String) -> bool {
        self.0.contains_key(fname)
    }

    pub fn get_and_cache(&mut self, fname: &String) -> Option<&Vec<u8>> {
        if self.contains_key(fname) {
            println!("{}", "Cache Hit.".green());
            self.get(fname)
        } else {
            println!("{}", "Cache Miss.".red());
            self.read_to_cache(fname);
            self.get(fname)
        }
    }
}