use std::collections::HashMap;
use std::fmt;

use crate::CaseInsensitiveString;
use crate::HttpMethod;
use crate::HttpRequest;
use crate::HttpResponse;
use crate::URI;
use crate::defaultfields::default_authorization;
use crate::defaultfields::{
    default_accept, default_accept_encoding, default_connection, default_content_length,
};

use crate::defaultmethods::{handle_get, handle_head, handle_options, handle_trace};

use aes_gcm::{
    Aes256Gcm,
    aead::{AeadCore, OsRng},
};

#[derive(Clone)]
pub struct Nonce {
    pub val: String,
    pub n: usize,
}

pub struct NonceTracker {
    pub map: HashMap<String, Nonce>,
}

impl NonceTracker {
    pub fn new() -> Self {
        NonceTracker {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String) {
        let nonce_crypt = Aes256Gcm::generate_nonce(&mut OsRng);
        let nonce = format!("{:x}", nonce_crypt);
        self.map.insert(name, Nonce { val: nonce, n: 0 });
    }

    pub fn get(&mut self, name: &String) -> Option<&mut Nonce> {
        let n = self.map.get_mut(name)?;
        n.n = n.n + 1;
        Some(n)
    }

    pub fn remove(&mut self, name: String) {
        self.map.remove(&name);
    }
}

pub struct UserAuth {
    pub user: String,
    pub pass: String,
    pub realm: String,
    pub nonce: Option<String>,
}

pub enum UserAuthResult {
    UNAUTHORIZED,
    AUTHORIZED,
    CHANGEREALM,
}

pub struct AuthData {
    pub method: HttpMethod,
    pub uri: URI,
    pub nonce: Option<String>,
}

pub union RequestState<'req> {
    pub path: &'req String,
    pub contents: &'req mut Vec<u8>,
    pub auth: &'req AuthData,
}

type HttpMethodHandler = dyn Fn(HttpRequest, &mut NonceTracker) -> HttpResponse + Sync + Send;

pub type WriterType<'req> =
    Option<Box<dyn FnMut(&[u8]) -> Result<usize, std::io::Error> + Send + Sync + 'req>>;
pub type DecoderType<'req> =
    Option<Box<dyn FnMut(&UserAuth) -> UserAuthResult + Send + Sync + 'req>>;

pub enum RequestEffect<'req> {
    WRITER(WriterType<'req>),
    DECODER(DecoderType<'req>),
}

// Take a single value from a header field and return a closure that updates the control flow of the current method
pub type HttpFieldHandler =
    dyn for<'req> Fn(String, &'req mut RequestState) -> RequestEffect<'req> + Send + Sync;

#[derive(Clone)]
pub struct HttpMethodHandlerTable(HashMap<HttpMethod, &'static HttpMethodHandler>);

impl fmt::Debug for HttpMethodHandlerTable {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct HttpFieldHandlerTable(HashMap<CaseInsensitiveString, &'static HttpFieldHandler>);

impl fmt::Debug for HttpFieldHandlerTable {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        Ok(())
    }
}

impl HttpMethodHandlerTable {
    pub fn new() -> Self {
        HttpMethodHandlerTable { 0: HashMap::new() }
    }

    pub fn insert(&mut self, key: HttpMethod, value: &'static HttpMethodHandler) {
        self.0.insert(key, value);
    }

    pub fn get(&self, key: HttpMethod) -> Option<&HttpMethodHandler> {
        self.0.get(&key).map(|v| &**v)
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
        HttpFieldHandlerTable { 0: HashMap::new() }
    }

    pub fn insert(&mut self, key: CaseInsensitiveString, value: &'static HttpFieldHandler) {
        self.0.insert(key, value);
    }

    pub fn get(&self, key: &CaseInsensitiveString) -> Option<&HttpFieldHandler> {
        self.0.get(key).map(|v| &**v)
    }

    pub fn use_defaults(&mut self) {
        self.insert(CaseInsensitiveString::from_str("accept"), &default_accept);
        self.insert(
            CaseInsensitiveString::from_str("accept-encoding"),
            &default_accept_encoding,
        );
        self.insert(
            CaseInsensitiveString::from_str("authorization"),
            &default_authorization,
        );
        self.insert(
            CaseInsensitiveString::from_str("content"),
            &default_connection,
        );
        self.insert(
            CaseInsensitiveString::from_str("content-length"),
            &default_content_length,
        );
    }
}
