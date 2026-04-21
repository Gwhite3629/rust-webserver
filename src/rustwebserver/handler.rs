use std::collections::HashMap;
use lazy_static::__Deref;

use crate::HttpRequest;
use crate::HttpResponse;
use crate::HttpMethod;

use crate::defaultmethods::{
    handle_get,
    handle_head,
    handle_options,
    handle_trace,
};

type HttpMethodHandler = dyn Fn(HttpRequest) -> HttpResponse + Sync + Send;

#[derive(Clone)]
pub struct HttpMethodHandlerTable (
    HashMap<HttpMethod, &'static HttpMethodHandler>,
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