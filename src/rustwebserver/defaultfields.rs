use std::fs::File;
use std::path::Path;

use crate::CaseInsensitiveString;
use crate::RequestEffect;
use crate::RequestState;

use flate2::write::GzEncoder;
use flate2::Compression;


#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum DefaultFields {
    ACCEPT,
    ACCEPTENCODING,
    CONNECTION,
    CONTENTLENGTH,
}

impl DefaultFields {
    pub fn from_string(field: CaseInsensitiveString) -> Option<DefaultFields> {
        if CaseInsensitiveString::from_str("accept") == field {
            return Some(DefaultFields::ACCEPT);
        } else if CaseInsensitiveString::from_str("accept-encoding") == field {
            return Some(DefaultFields::ACCEPTENCODING);
        } else if CaseInsensitiveString::from_str("connection") == field {
            return Some(DefaultFields::CONNECTION);
        } else if CaseInsensitiveString::from_str("content-length") == field {
            return Some(DefaultFields::CONTENTLENGTH);
        } else {
            return None;
        }
    }

    pub fn to_string(&self) -> CaseInsensitiveString {
        match self {
            DefaultFields::ACCEPT => CaseInsensitiveString("accept".to_string()),
            DefaultFields::ACCEPTENCODING => CaseInsensitiveString("accept-encoding".to_string()),
            DefaultFields::CONNECTION=> CaseInsensitiveString("connection".to_string()),
            DefaultFields::CONTENTLENGTH => CaseInsensitiveString("content-length".to_string()),
        }
    }
}

// type HttpFieldHandler = dyn Fn(String, RequestState) -> RequestEffect + Sync + Send;

pub fn default_accept(val: String, state: RequestState) -> Option<RequestEffect> {
    None
}

pub fn default_accept_encoding(val: String, state: RequestState) -> Option<RequestEffect> {
    if val.contains("gzip") {
        return Some(RequestEffect{writer: Box::new(GzEncoder::new(unsafe {state.contents}, Compression::default()))})
    } else {
        None
    }
}

pub fn default_connection(val: String, state: RequestState) -> Option<RequestEffect> {
    None
}

pub fn default_content_length(val: String, state: RequestState) -> Option<RequestEffect> {
    None
}