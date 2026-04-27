use std::collections::HashMap;
use std::ops::Deref;

use crate::HttpRequest;
use crate::HttpResponse;
use crate::HttpMethod;
use crate::HttpFields;
use crate::CaseInsensitiveString;

use crate::defaultmethods::{
    handle_get,
    handle_head,
    handle_options,
    handle_trace,
};

type HttpMethodHandler = dyn Fn(HttpRequest) -> HttpResponse + Sync + Send;

// Take a single value from a header field and return a closure that updates the control flow of the current method
type HttpFieldHandler = dyn Fn(String) -> Box<dyn FnOnce()>;

#[derive(Clone)]
pub struct HttpMethodHandlerTable (
    HashMap<HttpMethod, &'static HttpMethodHandler>,
);

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

    pub fn get(&self, key: CaseInsensitiveString) -> Option<&HttpFieldHandler> {
        self.0.get(
            &key
        ).map(|v|&**v)
    }

    pub fn dispatch(&self, headers: HttpFields) -> impl Iterator<Item = Box<dyn FnOnce()>> {
        
        let list: Vec<Box<dyn FnOnce()>> = Vec::new();
        
        for (key, val) in headers {
            let fun = match self.get(key) {
                Some(fun) => {
                    let f = (*fun)(val);
                },
                None => (),
            };
        }

        list.into_iter()
    }
}