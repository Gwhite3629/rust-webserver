use std::collections::HashMap;
use std::io;

use crate::HttpRequest;
use crate::HttpResponse;
use crate::HttpMethod;
use crate::CaseInsensitiveString;
use crate::defaultfields::{
    default_accept,
    default_accept_encoding,
    default_connection,
    default_content_length,
};

use crate::defaultmethods::{
    handle_get,
    handle_head,
    handle_options,
    handle_trace,
};


pub union RequestState<'req> {
    pub path: &'req String,
    pub contents: &'req mut Vec<u8>,
}

/*
pub struct RequestEffect {
    pub writer: Box<dyn FnMut(&[u8]) -> io::Result<usize>>,
}
*/

type HttpMethodHandler = dyn Fn(HttpRequest) -> HttpResponse + Sync + Send;

// Take a single value from a header field and return a closure that updates the control flow of the current method
pub type HttpFieldHandler = dyn for<'req> Fn(String, &'req mut RequestState) -> Box<dyn FnMut(&[u8]) -> io::Result<usize> + 'req> + Sync + Send;

#[derive(Clone)]
pub struct HttpMethodHandlerTable (
    HashMap<HttpMethod, &'static HttpMethodHandler>,
);

#[derive(Clone)]
pub struct HttpFieldHandlerTable (
    HashMap<CaseInsensitiveString, &'static HttpFieldHandler>,
);

impl HttpMethodHandlerTable {
    pub fn new() -> Self {
        HttpMethodHandlerTable { 0: HashMap::new() }
    }

    pub fn insert(&mut self, key: HttpMethod, value: &'static HttpMethodHandler) {
        self.0.insert(
            key,
            value,
        );
    }

    pub fn get(&self, key: HttpMethod) -> Option<&HttpMethodHandler> {
        self.0.get(
            &key
        ).map(|v|&**v)
    }

    pub fn use_defaults(&mut self) {
        self.insert(HttpMethod::GET, &handle_get);
        self.insert(HttpMethod::HEAD, &handle_head);
        self.insert(HttpMethod::OPTIONS, &handle_options);
        self.insert(HttpMethod::TRACE, &handle_trace);
    }
}

impl HttpFieldHandlerTable {
    pub fn new() -> Self {
        HttpFieldHandlerTable{ 0: HashMap::new() }
    }

    pub fn insert(&mut self, key: CaseInsensitiveString, value: &'static HttpFieldHandler) {
        self.0.insert(
            key,
            value,
        );
    }

    pub fn get(&self, key: &CaseInsensitiveString) -> Option<&HttpFieldHandler> {
        self.0.get(
            key
        ).map(|v|&**v)
    }

    pub fn use_defaults(&mut self) {
        self.insert(CaseInsensitiveString::from_str("accept"), &default_accept);
        self.insert(CaseInsensitiveString::from_str("accept-encoding"), &default_accept_encoding);
        self.insert(CaseInsensitiveString::from_str("content"), &default_connection);
        self.insert(CaseInsensitiveString::from_str("content-length"), &default_content_length);
    }
}