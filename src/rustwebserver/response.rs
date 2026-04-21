
use crate::HttpFields;
use crate::HttpStatus;

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
        let content: String = String::from_utf8(self.content.clone()).unwrap();

        let response: String = status + headers.as_str() + content.as_str();
        response
    }
}