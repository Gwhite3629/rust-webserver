use std::path::{Path, PathBuf, Component};

use crate::Auth;

use crate::CONFIG;

pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}

pub fn is_valid_path(path: &Path, name: &String) -> bool {
    if !path.has_root() {
        return false;
    };

    let base = Path::new(&CONFIG.get().unwrap().servers.get(name).unwrap().path);
    let full_path = base.join(path.strip_prefix("/").unwrap());

    println!("Full Path: {full_path:#?}");

    if !full_path.is_file() {
        return false;
    };

    let resolved_base = normalize_path(base);
    let resolved_path = normalize_path(full_path.as_path());

    if !resolved_path.starts_with(resolved_base) {
        return false;
    };

    println!("Final Path: {resolved_path:#?}");

    return true;
}

pub fn resolve_path(req_path: &Path, name: &String) -> (PathBuf, Option<Auth>) {
    let mut res_path = req_path.to_path_buf();

    let mut set_auth = false;
    let mut auth: Option<Auth> = None;

    let root_redirect = CONFIG
        .get()
        .unwrap()
        .servers
        .get(name)
        .unwrap()
        .root_redirect
        .clone();
    let redirects = CONFIG
        .get()
        .unwrap()
        .servers
        .get(name)
        .unwrap()
        .redirects
        .clone();

    match root_redirect {
        Some(red) => {
            res_path = red
                .redirect
                .unwrap()
                .join(res_path.strip_prefix("/").unwrap());
            match red.auth {
                Some(a) => {
                    set_auth = true;
                    auth = Some(a);
                }
                None => auth = None,
            };
        }
        None => (),
    }

    match redirects {
        Some(reds) => {
            for r in reds {
                if set_auth == false {
                    if res_path
                        .to_string_lossy()
                        .to_string()
                        .contains(r.req_path.to_str().unwrap())
                    {
                        match r.auth {
                            Some(a) => {
                                set_auth = true;
                                auth = Some(a);
                            },
                            None => auth = None,
                        };
                    }
                }
                match r.redirect {
                    Some(direct) => {
                        res_path = PathBuf::from(res_path.to_string_lossy().replacen(
                            r.req_path.to_str().unwrap(),
                            direct.to_str().unwrap(),
                            1,
                        ));
                    },
                    None => (),
                }
            }
        }
        None => (),
    }

    (res_path, auth)
}

pub fn get_mimetype(file: &String) -> String {
    let e = Path::new(file.as_str())
        .extension()
        .map(|ext| ext.to_str().unwrap_or(""))
        .unwrap_or("");

    let mimetype = match e {
        // Text
        "html" | "htm" => "text/html; charset=utf-8",
        "js" => "text/javascript",
        "mjs" => "text/javascript",
        "css" => "text/css",
        "json" => "application/json",
        "xml" => "application/xml",
        "txt" => "text/plain",
        "md" => "text/markdown",
        "csv" => "text/csv",

        // Images
        "ico" => "image/x-icon",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "tiff" | "tif" => "image/tiff",

        // Audio
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "flac" => "audio/flac",

        // Video
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "avi" => "video/x-msvideo",
        "mkv" => "video/x-matroska",

        // Documents
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",

        // Archives
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",
        "7z" => "application/x-7z-compressed",
        "rar" => "application/vnd.rar",

        // Fonts
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "woff" => "font/woff",
        "woff2" => "font/woff2",

        // For unknown types, use a safe default
        _ => "application/octet-stream",
    };

    mimetype.to_string()
}
