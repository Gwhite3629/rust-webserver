use std::io::Write;
use std::io::BufWriter;

use crate::CaseInsensitiveString;
use crate::RequestState;
use crate::RequestEffect;

use flate2::write::GzEncoder;
use flate2::Compression;


#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum DefaultFields {
    ACCEPT,
    ACCEPTENCODING,
    AUTHORIZATION,
    CONNECTION,
    CONTENTLENGTH,
}

impl DefaultFields {
    pub fn from_string(field: CaseInsensitiveString) -> Option<DefaultFields> {
        if CaseInsensitiveString::from_str("accept") == field {
            return Some(DefaultFields::ACCEPT);
        } else if CaseInsensitiveString::from_str("accept-encoding") == field {
            return Some(DefaultFields::ACCEPTENCODING);
        } else if CaseInsensitiveString::from_str("authorization") == field {
            return Some(DefaultFields::AUTHORIZATION);
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
            DefaultFields::AUTHORIZATION => CaseInsensitiveString("authorization".to_string()),
            DefaultFields::CONNECTION=> CaseInsensitiveString("connection".to_string()),
            DefaultFields::CONTENTLENGTH => CaseInsensitiveString("content-length".to_string()),
        }
    }
}

// type HttpFieldHandler = dyn Fn(String, RequestState) -> RequestEffect + Sync + Send;

pub fn default_accept<'req>(_val: String, _state: &'req mut RequestState) -> RequestEffect<'req>
{
    todo!()
}

pub fn default_accept_encoding<'req>(val: String, state: &'req mut RequestState) -> RequestEffect<'req>
{    
    if val.contains("gzip") {
        let mut gz = GzEncoder::new(unsafe {&mut state.contents}, Compression::default());
        println!("Using gzip encoding");
        return RequestEffect::WRITER(
            Some(
                Box::new(
                    move |f| gz.write(f.as_ref())
                )
            )
        );
    } else {
        let mut bf = BufWriter::new(unsafe {&mut state.contents});
        println!("Using identity encoding");
        return RequestEffect::WRITER(
            Some(
                Box::new(
                    move |f| bf.write(f.as_ref())
                )
            )
        );
    };
}

pub fn default_authorization<'req>(val: String, state: &'req mut RequestState) -> RequestEffect<'req>
{
    if val.contains("Basic") {
        todo!();
    } else if val.contains("Digest") {
        todo!();
    } else {
        todo!();
    }
}

pub fn default_connection<'req>(_val: String, _state: &'req mut RequestState) -> RequestEffect<'req>
{
    todo!()
}

pub fn default_content_length<'req>(_val: String, _state: &'req mut RequestState) -> RequestEffect<'req>
{
    todo!()
}