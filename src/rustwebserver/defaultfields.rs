use std::io::Write;
use std::io::BufWriter;

use crate::CaseInsensitiveString;
use crate::RequestState;
use crate::RequestEffect;
use crate::handler::UserAuthResult;

use flate2::write::GzEncoder;
use flate2::Compression;

use base64::{Engine as _, engine::general_purpose::URL_SAFE};


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

// Receive authorization from user, basic or digest
pub fn default_authorization<'req>(val: String, state: &'req mut RequestState) -> RequestEffect<'req>
{
    // Receive basic auth
    // Simply decode from base64 and split at ':'
    if val.contains("Basic") {
        let str = val.split_once("Basic").unwrap().1.trim().to_owned();
        RequestEffect::DECODER(
            Some(
                Box::new(
                    move |a| {
                        match URL_SAFE.encode(a.user.clone() + ":" + a.pass.as_str()) == str {
                            true => UserAuthResult::AUTHORIZED,
                            false => UserAuthResult::UNAUTHORIZED,
                        }
                    }
                )
            )
        )
    // Digest is more complex
    // HA1 = MD5(user:realm:pass)
    // HA2 = MD5(method(GET):URI)
    // Response = MD5(HA1:servernonce:nc:cnonce:qop:HA2)
    // Client response should equal this value
    } else if val.contains("Digest") {
        let (_, rest) = val.split_once("Digest").unwrap();
        let rest = rest.trim().to_owned();
        let client_vals: Vec<&str> = rest.split(",").map(|p| p.trim()).collect();
        let pairs: Vec<(&str,&str)> = client_vals.iter().map(|p| p.split_once("=").unwrap()).collect();
        let mut nc = String::new();
        let mut cnonce = String::new();
        let mut qop = String::new();
        let mut client_response = String::new();
        let mut realm = String::new();
        for (left, right) in pairs {
            match left {
                "nc" => nc = String::from(right),
                "cnonce" => {
                    cnonce = String::from(right);
                    cnonce.retain(|c| c != '\"');
                },
                "qop" => qop = String::from(right),
                "response" => {
                    client_response = String::from(right);
                    client_response.retain(|c| c != '\"');
                },
                "realm" => {
                    realm = String::from(right);
                    realm.retain(|c| c != '\"');
                },
                _ => (),
            }
        }
        RequestEffect::DECODER(
            Some(
                Box::new(
                    move |a| {
                        if realm != a.realm.as_str() {
                            UserAuthResult::CHANGEREALM
                        } else {
                            let ha1: String = format!("{:x}",md5::compute(a.user.clone() + ":" + a.realm.as_str() + ":" + a.pass.as_str()));
                            let ha2: String = format!("{:x}",md5::compute(unsafe{state.auth.method.as_str()}.to_owned() + ":" + &unsafe{state.auth.uri.to_string()}));
                            // Response = MD5(HA1:servernonce:nc:cnonce:qop:HA2)
                            let response: String = format!("{:x}",
                                md5::compute(ha1 + ":" + 
                                    match &a.nonce {
                                        Some(v) => v,
                                        None => "0",
                                    } + ":"
                                    + nc.as_str() + ":" + cnonce.as_str() + ":" + qop.as_str() + ":" + ha2.as_str()));
                            match response == client_response {
                                true => UserAuthResult::AUTHORIZED,
                                false => UserAuthResult::UNAUTHORIZED,
                            }
                        }
                    }
                )
            )
        )
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