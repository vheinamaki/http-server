use crate::server::ContentHeaders;
use chrono::Utc;
use flate2::{write::GzEncoder, Compression};
use std::collections::HashMap;
use std::io::Result;
use std::io::Write;
use std::net::TcpStream;

pub enum HttpStatus {
    Ok,
    NotFound,
    BadRequest,
    NotAllowed,
    ServerError,
    UnsupportedVersion,
}

/// Struct representing a HTTP Response
pub struct Response<'a> {
    pub status: HttpStatus,
    pub protocol: String,
    pub headers: HashMap<&'a str, String>,
    pub payload: Vec<u8>,
}

impl<'a> Response<'a> {
    /// Returns a new HTTP/1.1 response with the given status, payload and empty headers.
    ///
    /// # Arguments
    /// * `status` - The response's HTTP status code
    /// * `payload` - The response's data
    pub fn new(status: HttpStatus, payload: Vec<u8>) -> Self {
        Response {
            status,
            protocol: String::from("HTTP/1.1"),
            headers: HashMap::new(),
            payload,
        }
    }

    fn status_to_string(&self) -> &str {
        match &self.status {
            HttpStatus::Ok => "200 OK",
            HttpStatus::NotFound => "404 NOT FOUND",
            HttpStatus::BadRequest => "400 BAD REQUEST",
            HttpStatus::NotAllowed => "405 METHOD NOT ALLOWED",
            HttpStatus::ServerError => "500 INTERNAL SERVER ERROR",
            HttpStatus::UnsupportedVersion => "505 HTTP VERSION NOT SUPPORTED",
        }
    }

    fn headers_to_string(&self) -> String {
        let mut result = String::new();
        result.push_str(&format!(
            "{} {}\r\n",
            &self.protocol,
            &self.status_to_string()
        ));
        for kv in &self.headers {
            let line = format!("{}: {}\r\n", kv.0, kv.1);
            result.push_str(&line);
        }
        result.push_str("\r\n");
        result
    }

    /// Add common headers to the response.
    pub fn set_default_headers(&mut self) {
        let length = self.payload.len().to_string();
        let time = Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        self.headers.insert("Connection", String::from("close"));
        self.headers.insert("Content-Length", length);
        self.headers.insert("Date", time);
        self.headers
            .insert("Permissions-Policy", "interest-cohort=()".to_string());
    }

    /// Add content type specific headers to the response.
    pub fn set_content_headers(&mut self, headers: &ContentHeaders) {
        let cache_control = format!("max-age={}", headers.cache_age);

        self.headers
            .insert("Content-Type", headers.content_type.to_string());
        self.headers.insert("Cache-Control", cache_control);
    }

    /// Compress the response payload using gzip, and set the correct encoding headers.
    pub fn compress_gzip(&mut self) -> Result<()> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&self.payload)?;
        self.payload = encoder.finish()?;

        self.headers.insert("Content-Encoding", "gzip".to_string());
        self.headers.insert("Vary", "Accept-Encoding".to_string());
        // Update content length
        self.headers
            .insert("Content-Length", self.payload.len().to_string());
        Ok(())
    }

    /// Write the response to the given `TcpStream`.
    pub fn send(&self, stream: &mut TcpStream) -> Result<()> {
        stream.write(&self.headers_to_string().as_bytes())?;
        stream.write_all(&self.payload)?;
        stream.flush()?;
        Ok(())
    }
}
