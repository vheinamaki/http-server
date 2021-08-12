use chrono::Utc;
use crate::server::ContentHeaders;
use std::net::TcpStream;
use std::io::Write;
use std::collections::HashMap;

pub enum HttpStatus {
    Ok,
    NotFound,
    BadRequest,
    ServerError,
}

pub struct Response<'a> {
    pub status: HttpStatus,
    pub protocol: String,
    pub headers: HashMap<&'a str, String>,
    pub payload: &'a [u8]
}

impl<'a> Response<'a> {
    pub fn new(status: HttpStatus, payload: &'a [u8]) -> Self {
        Response {
            status,
            protocol: String::from("HTTP/1.1"),
            headers: HashMap::new(),
            payload
        }
    }

    fn status_to_string(&self) -> &str {
        match &self.status {
            HttpStatus::Ok => "200 OK",
            HttpStatus::NotFound => "404 NOT FOUND",
            HttpStatus::BadRequest => "400 BAD REQUEST",
            HttpStatus::ServerError => "500 INTERNAL SERVER ERROR",
        }
    }

    fn headers_to_string(&self) -> String {
        let mut result = String::new();
        result.push_str(&format!("{} {}\r\n", &self.protocol, &self.status_to_string()));
        for kv in &self.headers {
            let line = format!("{}: {}\r\n", kv.0, kv.1);
            result.push_str(&line);
        }
        result.push_str("\r\n");
        result
    }

    pub fn set_default_headers(&mut self) {
        let length = self.payload.len().to_string();
        let time = Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        self.headers.insert("Content-Length", length);
        self.headers.insert("Date", time);
        self.headers.insert("Permissions-Policy", "interest-cohort=()".to_string());
    }

    pub fn set_content_headers(&mut self, headers: &ContentHeaders) {
        let cache_control = format!("max-age={}", headers.cache_age);

        self.headers.insert("Content-Type", headers.content_type.to_string());
        self.headers.insert("Cache-Control", cache_control);
    }

    pub fn send(&self, stream: &mut TcpStream) -> Result<(), std::io::Error> {
        stream.write(&self.headers_to_string().as_bytes())?;
        stream.write(&self.payload)?;
        stream.flush()?;
        Ok(())
    }
}