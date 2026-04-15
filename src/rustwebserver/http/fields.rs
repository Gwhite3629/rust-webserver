use std::collections::{hash_map, HashMap};
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

#[derive(Clone, Debug)]
pub struct CaseInsensitiveString(String);

impl PartialEq for CaseInsensitiveString {
    fn eq (&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}
impl Eq for CaseInsensitiveString {}

impl Display for CaseInsensitiveString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Hash for CaseInsensitiveString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for b in self.0.bytes() {
            state.write_u8(b.to_ascii_lowercase());
        }
    }
}

impl Deref for CaseInsensitiveString {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Default, Debug)]
pub struct HttpFields {
    fields: HashMap<CaseInsensitiveString, String>,
}

impl HttpFields {
    pub fn new() -> Self {
        HttpFields { fields: HashMap::new() }
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        self.fields.insert(
            CaseInsensitiveString(key.to_string()),
            value.to_string()
        );
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.fields.get(
            &CaseInsensitiveString(key.to_string())
        )
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, CaseInsensitiveString, String> {
        self.fields.iter()
    }
}

impl IntoIterator for HttpFields {
    type Item = (CaseInsensitiveString, String);
    type IntoIter = hash_map::IntoIter<CaseInsensitiveString, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.into_iter()
    }
}

impl Display for HttpFields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (k,v) in self.iter() {
            write!(f,"{:#?}: {:#?}\n", k, v)?
        }
        write!(f, "\n")
    }
}