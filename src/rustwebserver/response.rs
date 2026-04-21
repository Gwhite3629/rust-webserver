use std::fmt::Display;

use lazy_static::lazy_static;

use crate::HttpFields;
use crate::HttpStatus;

#[derive(Debug, Default)]
pub struct HttpResponse {
    pub version: String,
    pub status: HttpStatus,
    pub headers: HttpFields,
    pub content: Vec<u8>,
}

/*
    let (status_line, filename) = match http_request[0].as_str() {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = match fs::read_to_string(filename) {
        Ok(contents) => contents,
        Err(error) => panic!("Error reading file: {error:?}"),
    };
    let length = contents.len();

    let response = format!(
        "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
    ); 
*/

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
        let content: String = String::from_utf8(self.content.clone()).unwrap();

        let response: String = status + headers.as_str() + content.as_str();
        response
    }
}