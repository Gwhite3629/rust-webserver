use std::fmt::Display;

use lazy_static::lazy_static;

use crate::HttpFields;
use crate::HttpStatus;

#[derive(Debug)]
pub struct HttpResponse {
    pub status: HttpStatus,
    pub headers: HttpFields,
    pub content: Vec<u8>,
}