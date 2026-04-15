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
            .split(' ').map(|s| s.trim())     // (2)
            .filter(|s| !s.is_empty())        // (3)
            .map(|s| s.parse().unwrap())      // (4)
            .collect();                       // (5)
        return formatted;
    }

    fn format_host(host: String) -> Vec<String> {
        let formatted: Vec<String> = host
            .split(':').map(|s| s.trim())     // (2)
            .filter(|s| !s.is_empty())        // (3)
            .map(|s| s.parse().unwrap())      // (4)
            .collect();

        return formatted;
    }

    fn format_headers(request: Vec<String>) -> HttpFields {
        let mut headers: HttpFields = HttpFields::new();

        for req in request.iter() {
            let (rkey, rval) = req.split_once(':').unwrap();
            headers.insert(rkey, rval);
        }

        return headers;
    }
}

impl Display for HttpRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Method: {}", self.method.to_str())?;
        write!(f, "URI: {}", self.target.to_string())?;
        write!(f, "Headers: {}", self.headers.to_string())?;
        write!(f, "Content: {:#?}", self.content)
    }
}