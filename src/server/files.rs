use std::fs::File;
use std::io::Read;
use std::io::Result;
use std::path::Path;

pub struct HttpContent {
    file_path: String,
}

impl HttpContent {
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

    pub fn get_bytes(&self) -> Result<Vec<u8>> {
        let mut file = File::open(Path::new(&self.file_path))?;
        let mut buffer = Vec::<u8>::new();
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

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

        let (ctype, age) = match ext {
            "html" => ("text/html; charset=UTF-8", min),
            "css" => ("text/css; charset=UTF-8", 3 * day),
            "js" => ("text/javascript; charset=UTF-8", 3 * day),
            "txt" => ("text/plain; charset=UTF-8", min),
            "json" => ("application/json; charset=UTF-8", hour),
            "svg" => ("image/svg+xml; charset=UTF-8", 7 * day),
            "webp" => ("image/webp", 3 * day),
            "jpg" | "jpeg" => ("image/jpeg", 3 * day),
            "ico" => ("image/x-icon", 7 * day),
            "png" => ("image/png", 3 * day),
            "otf" => ("font/otf", 7 * day),
            "ttf" => ("font/ttf", 7 * day),
            "mp4" => ("video/mp4", day),
            "mp3" => ("audio/mp3", day),
            _ => ("application/octet-stream", min),
        };
        ContentHeaders {
            content_type: ctype,
            cache_age: age
        }
    }
}

pub struct ContentHeaders<'a> {
    pub content_type: &'a str,
    pub cache_age: u32
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
