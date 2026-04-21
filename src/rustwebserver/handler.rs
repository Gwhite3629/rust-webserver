use std::collections::HashMap;

use crate::HttpRequest;
use crate::HttpResponse;
use crate::HttpMethod;

type HttpMethodHandler = dyn Fn(HttpRequest) -> HttpResponse;

pub struct HttpMethodHandlerTable (
    HashMap<HttpMethod, Box<HttpMethodHandler>>,
);

impl HttpMethodHandlerTable {
    pub fn new() -> Self {
        HttpMethodHandlerTable { 0: HashMap::new() }
    }

    pub fn insert(&mut self, key: HttpMethod, value: &'static HttpMethodHandler) {
        self.0.insert(
            key,
            Box::new(value),
        );
    }

    pub fn get(&self, key: HttpMethod) -> Option<&Box<HttpMethodHandler>> {
        self.0.get(
            &key
        )
    }
}