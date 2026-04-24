//use std::fs::metadata;

use crate::HttpFields;
use crate::HttpStatus;
use crate::file::get_mimetype;

#[derive(Debug, Default)]
pub struct HttpResponse {
    pub version: String,
    pub status: HttpStatus,
    pub headers: HttpFields,
    pub content: Vec<u8>,
}

impl HttpResponse {
    pub fn new() -> Self {
        HttpResponse::default()
    }

    pub fn to_string(&self) -> String {
        let status: String = String::new() + self.version.as_str() + " " + (self.status as u16).to_string().as_str() + " " + self.status.to_string() + "\r\n";
        let mut headers: String = String::new();
        for (k, v) in self.headers.clone() {
            headers = headers + &k + ": " + v.as_str() + "\r\n";
        }
        headers = headers + "\r\n\r\n";

        let response: String = status + headers.as_str();
        response
    }

    pub fn generate_get_headers(file: String, len: usize) -> HttpFields {
        let mut res = HttpFields::new();

        //let metadata = metadata(&file).unwrap();

        res.insert("content-length", len.to_string().as_str());
        res.insert("content-type", get_mimetype(file).as_str());
        res.insert("content-encoding", "gzip");
        res.insert("transfer-encoding", "chunked");

        res
    }

    pub fn generate_trace_headers(len: usize) -> HttpFields {
        let mut res = HttpFields::new();

        //let metadata = metadata(&file).unwrap();

        res.insert("content-length", len.to_string().as_str());
        res.insert("content-encoding", "gzip");
        res.insert("transfer-encoding", "chunked");

        res
    }
}