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

pub fn get_mimetype(file: String) -> String {

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