use std::path::Path;

use crate::CONFIG;

pub fn is_valid_path(path: &Path) -> bool {
    if !path.has_root() {return false};

    let base = Path::new(&CONFIG.get().unwrap().path);
    let full_path = base.join(path.strip_prefix("/").unwrap());

    if !full_path.is_file() {return false};

    let resolved_base = base.canonicalize().unwrap();
    let resolved_path = full_path.canonicalize().unwrap();

    if !resolved_path.starts_with(resolved_base) {return false};

    return true;
}