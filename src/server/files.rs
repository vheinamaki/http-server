use std::fs::File;
use std::io::Read;
use std::io::Result;
use std::path::Path;

/// Represents a file in the served folder
pub struct HttpContent {
    file_path: String,
}

impl HttpContent {

    /// Returns a new HttpContent instance for the given folder and file path.
    /// Returns `None` if the file does not exist in the served folder.
    ///
    /// # Arguments
    ///
    /// * `serve_path` - The served folder
    /// * `content_path` - Path of the requested file, relative to `serve_path`
    pub fn new(serve_path: &str, content_path: &str) -> Option<Self> {
        let content_path = content_path.strip_prefix(&['/', '\\'][..]).unwrap_or(content_path);
        let combined_path = format!("{}/{}", serve_path, content_path);
        let file_path = resolve_file_path(combined_path.to_string())?;
        if in_serve_folder(serve_path, &file_path) {
            Some(HttpContent { file_path })
        } else {
            None
        }
    }

    /// Returns the file's contents as a byte vector.
    /// Returns `io::Error` if the file could not be read.
    pub fn get_bytes(&self) -> Result<Vec<u8>> {
        let mut file = File::open(Path::new(&self.file_path))?;
        let mut buffer = Vec::<u8>::new();
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    /// Return the content type specific response headers for the file.
    pub fn content_headers(&self) -> ContentHeaders {
        let ext = match Path::new(&self.file_path).extension() {
            Some(x) => match x.to_str() {
                Some(y) => y,
                None => "",
            },
            None => "",
        };
        let min = 60;
        let hour = 3600;
        let day = 86400;

        let (ctype, age, use_gzip) = match ext {
            "html" => ("text/html; charset=UTF-8", min, true),
            "css" => ("text/css; charset=UTF-8", 3 * day, true),
            "js" => ("text/javascript; charset=UTF-8", 3 * day, true),
            "txt" => ("text/plain; charset=UTF-8", min, true),
            "json" => ("application/json; charset=UTF-8", hour, true),
            "svg" => ("image/svg+xml; charset=UTF-8", 7 * day, true),
            "webp" => ("image/webp", 3 * day, false),
            "jpg" | "jpeg" => ("image/jpeg", 3 * day, false),
            "ico" => ("image/x-icon", 7 * day, false),
            "png" => ("image/png", 3 * day, false),
            "otf" => ("font/otf", 7 * day, true),
            "ttf" => ("font/ttf", 7 * day, true),
            "mp4" => ("video/mp4", day, false),
            "mp3" => ("audio/mp3", day, false),
            _ => ("application/octet-stream", min, false),
        };
        ContentHeaders {
            content_type: ctype,
            cache_age: age,
            compress: use_gzip
        }
    }
}

/// Struct representing a file's content type specific response headers.
/// * `content_type` - The file's Content-Type header value
/// * `cache_age` - The file's Cache-Control: max-age value
/// * `compress` - Whether the file should be compressed with gzip
pub struct ContentHeaders<'a> {
    pub content_type: &'a str,
    pub cache_age: u32,
    pub compress: bool
}

fn resolve_file_path(path: String) -> Option<String> {
    let mut validated = String::from(path);
    let path = Path::new(&validated);
    if path.is_file() {
        return Some(validated);
    } else if path.extension() == None && path.with_extension("html").is_file() {
        validated.push_str(".html");
        return Some(validated);
    } else if path.is_dir() && path.join("index.html").is_file() {
        match path.join("index.html").to_str() {
            Some(x) => return Some(String::from(x)),
            None => return None,
        }
    }
    None
}

fn in_serve_folder(root: &str, path: &str) -> bool {
    let root = match Path::new(root).canonicalize() {
        Ok(value) => value,
        Err(_) => {
            return false;
        }
    };
    let path = Path::new(path);
    let canonicalized = match path.canonicalize() {
        Ok(value) => value,
        Err(_) => {
            return false;
        }
    };
    if !canonicalized.starts_with(root) {
        return false;
    }
    true
}
