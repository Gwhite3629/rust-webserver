use std::fmt::Display;

use lazy_static::lazy_static;

use crate::HttpFields;
use crate::HttpMethod;
use crate::URI;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub server_name: String,
    pub method: HttpMethod,
    pub target: URI,
    pub version: String,
    pub headers: HttpFields,
    pub content: Vec<u8>,
}

impl HttpRequest {
    pub fn new (request: Vec<String>, name: String) -> Self {
        let form_s = Self::format_scheme(request[0].clone());
        //let form_h = Self::format_host(request[1].clone());
        return HttpRequest { 
            server_name: name,
            method: HttpMethod::from_str(&form_s[0]).unwrap(),
            target: URI::new(&form_s[1]),
            version: form_s[2].clone(),
            headers: HttpFields::populate(request),
            content: Vec::new(),
        }
    }

    // First line of request always has {METHOD' 'URI' 'VERSION}
    fn format_scheme(scheme: String) ->  Vec<String> {
        let formatted: Vec<String> = scheme
            .split(' ').map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.parse().unwrap())
            .collect();

        return formatted;
    }

    fn _format_host(host: String) -> Vec<String> {
        let formatted: Vec<String> = host
            .split(':').map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.parse().unwrap())
            .collect();

        for f in formatted.iter() {
            println!("host line: {}", f);
        }

        return formatted;
    }

    fn _insert_content(&mut self, v: Vec<u8>) {
        self.content = v;
    }

    pub fn to_string(&self) -> String {
        let status: String = String::new() + 
            self.method.as_str() + " " + 
            self.target.to_string().as_str() + " " + 
            self.version.as_str() + "\r\n";
        let mut headers: String = String::new();
        for (k, v) in self.headers.clone() {
            headers = headers + &k + ": " + v.as_str() + "\r\n";
        }
        headers = headers + "\r\n\r\n";

        let response: String = status + headers.as_str();
        response
    }
}

impl Display for HttpRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Method: {}\n", self.method.as_str())?;
        write!(f, "URI: \n{}\n", self.target.to_string())?;
        write!(f, "Version: {}\n", self.version)?;
        write!(f, "Headers: \n{}\n", self.headers.to_string())?;
        for b in self.content.iter() {
            write!(f,"{:x?}", b)?
        }
        write!(f, "\n")
    }
}

// TESTS

lazy_static! {
    static ref RAW_BASIC_REQUEST: String = String::from("GET / HTTP/1.1\r\nHost: 127.0.0.1:7878\r\n\r\n");
}
#[cfg(test)]
#[test]
fn basic_request() {
    let raw_request = RAW_BASIC_REQUEST
    .lines()
    .map(|res| res.to_string())
    .take_while(|line| !line.is_empty())
    .collect();

    let http_request: HttpRequest = HttpRequest::new(raw_request, "default".to_string());

    assert_eq!(http_request.method.as_str(),"GET");

    assert_eq!(http_request.target.scheme.as_str(),"");
    assert_eq!(http_request.target.authority.userinfo.as_str(),"");
    assert_eq!(http_request.target.authority.host.as_str(),"");
    assert_eq!(http_request.target.authority.port,0);
    assert_eq!(http_request.target.path.as_str(),"/");
    assert_eq!(http_request.target.query.as_str(),"");
    assert_eq!(http_request.target.fragment.as_str(),"");

    assert_eq!(http_request.version.as_str(),"HTTP/1.1");

    assert_eq!(http_request.headers.get("Host").unwrap(),"127.0.0.1:7878");

    assert!(http_request.content.is_empty());
}

lazy_static! {
    static ref RAW_POST_REQUEST: String = String::from("POST /foo HTTP/1.1\r\nHost: 127.0.0.1:7878\r\nContent-length: 21\r\n\r\n");
}
lazy_static! {
    static ref RAW_POST_CONTENT: Vec<u8> = ("This is 21 characters").as_bytes().to_vec();
}
#[cfg(test)]
#[test]
fn post_request() {
    let raw_request = RAW_POST_REQUEST
    .lines()
    .map(|res| res.to_string())
    .take_while(|line| !line.is_empty())
    .collect();

    let mut http_request: HttpRequest = HttpRequest::new(raw_request, "default".to_string());

    http_request._insert_content(RAW_POST_CONTENT.to_vec());

    let http_string: String = String::from_utf8(http_request.content).unwrap();

    assert_eq!(http_request.method.as_str(),"POST");

    assert_eq!(http_request.target.scheme.as_str(),"");
    assert_eq!(http_request.target.authority.userinfo.as_str(),"");
    assert_eq!(http_request.target.authority.host.as_str(),"");
    assert_eq!(http_request.target.authority.port,0);
    assert_eq!(http_request.target.path.as_str(),"/foo");
    assert_eq!(http_request.target.query.as_str(),"");
    assert_eq!(http_request.target.fragment.as_str(),"");

    assert_eq!(http_request.version.as_str(),"HTTP/1.1");

    assert_eq!(http_request.headers.get("Host").unwrap(),"127.0.0.1:7878");
    assert_eq!(http_request.headers.get("Content-length").unwrap(),"21");

    assert_eq!(http_string,"This is 21 characters");
}


lazy_static! {
    static ref RAW_FULL_REQUEST: String = String::from("POST http://user:pass@www.example.com:6969/foo?key=value#frag HTTP/1.1\r\nHost: 127.0.0.1:7878\r\nContent-length: 21\r\n\r\n");
}
lazy_static! {
    static ref RAW_FULL_CONTENT: Vec<u8> = ("This is 21 characters").as_bytes().to_vec();
}
#[cfg(test)]
#[test]
fn full_uri() {
    let raw_request = RAW_FULL_REQUEST
    .lines()
    .map(|res| res.to_string())
    .take_while(|line| !line.is_empty())
    .collect();

    let mut http_request: HttpRequest = HttpRequest::new(raw_request, "default".to_string());

    http_request._insert_content(RAW_FULL_CONTENT.to_vec());

    let http_string: String = String::from_utf8(http_request.content).unwrap();

    assert_eq!(http_request.method.as_str(),"POST");

    assert_eq!(http_request.target.scheme.as_str(),"http");
    assert_eq!(http_request.target.authority.userinfo.as_str(),"user:pass");
    assert_eq!(http_request.target.authority.host.as_str(),"www.example.com");
    assert_eq!(http_request.target.authority.port,6969);
    assert_eq!(http_request.target.path.as_str(),"/foo");
    assert_eq!(http_request.target.query.as_str(),"key=value");
    assert_eq!(http_request.target.fragment.as_str(),"frag");

    assert_eq!(http_request.version.as_str(),"HTTP/1.1");

    assert_eq!(http_request.headers.get("Host").unwrap(),"127.0.0.1:7878");
    assert_eq!(http_request.headers.get("Content-length").unwrap(),"21");

    assert_eq!(http_string,"This is 21 characters");
}

lazy_static! {
    static ref RAW_BAD_REQUEST: String = String::from("POST www.example.com:6969/foo?key=value#frag HTTP/1.1\r\nHost: 127.0.0.1:7878\r\nContent-length: 21\r\n\r\n");
}
lazy_static! {
    static ref RAW_BAD_CONTENT: Vec<u8> = ("This is 21 characters").as_bytes().to_vec();
}
#[cfg(test)]
#[test]
fn improper_uri() {
    let raw_request = RAW_BAD_REQUEST
    .lines()
    .map(|res| res.to_string())
    .take_while(|line| !line.is_empty())
    .collect();

    let mut http_request: HttpRequest = HttpRequest::new(raw_request, "default".to_string());

    http_request._insert_content(RAW_BAD_CONTENT.to_vec());

    let http_string: String = String::from_utf8(http_request.content).unwrap();

    assert_eq!(http_request.method.as_str(),"POST");

    assert_eq!(http_request.target.scheme.as_str(),"www.example.com");
    assert_eq!(http_request.target.authority.userinfo.as_str(),"");
    assert_eq!(http_request.target.authority.host.as_str(),"");
    assert_eq!(http_request.target.authority.port,0);
    assert_eq!(http_request.target.path.as_str(),"6969/foo");
    assert_eq!(http_request.target.query.as_str(),"key=value");
    assert_eq!(http_request.target.fragment.as_str(),"frag");

    assert_eq!(http_request.version.as_str(),"HTTP/1.1");

    assert_eq!(http_request.headers.get("Host").unwrap(),"127.0.0.1:7878");
    assert_eq!(http_request.headers.get("Content-length").unwrap(),"21");

    assert_eq!(http_string,"This is 21 characters");
}