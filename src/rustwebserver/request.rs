use std::fmt::Display;

use crate::HttpFields;
use crate::HttpMethod;
use crate::URI;

#[derive(Debug)]
pub struct HttpRequest {
    method: HttpMethod,
    target: URI,
    headers: HttpFields,
    content: Vec<u8>,
}

impl HttpRequest {
    pub fn new (request: Vec<String>) -> Self {
        let form_s = Self::format_scheme(request[0].clone());
        let form_h = Self::format_host(request[1].clone());
        return HttpRequest { 
            method: HttpMethod::from_str(&form_s[0]),
            target: URI::new(form_s, form_h), 
            headers: Self::format_headers(request),
            content: Vec::new(),
        }
    }

    fn format_scheme(scheme: String) ->  Vec<String> {
        let formatted: Vec<String> = scheme
            .split(' ').map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.parse().unwrap())
            .collect();

        for f in formatted.iter() {
            println!("scheme line: {}", f);
        }

        return formatted;
    }

    fn format_host(host: String) -> Vec<String> {
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

    fn format_headers(request: Vec<String>) -> HttpFields {
        let mut headers: HttpFields = HttpFields::new();

        for req in request.iter() {
            let () = match req.split_once(": ") {
                Some((rkey, rval)) => headers.insert(rkey, rval),
                None => (),
            };
        }

        return headers;
    }
}

impl Display for HttpRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Method: {}\n", self.method.to_str())?;
        write!(f, "URI: \n{}\n", self.target.to_string())?;
        write!(f, "Headers: \n{}\n", self.headers.to_string())?;
        for b in self.content.iter() {
            write!(f,"{:x?}", b)?
        }
        write!(f, "\n")
    }
}