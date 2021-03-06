use chrono::Local;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

pub mod server;

use server::{ContentHeaders, HttpContent, HttpStatus, Request, Response, ThreadPool};

pub struct Arguments {
    pub directory: String,
    pub port: u16,
    pub threads: usize,
}

pub enum LogLevel {
    Info,
    ClientError,
    ServerError,
}

pub fn log(msg: &str, level: LogLevel) {
    let (id, color) = match level {
        LogLevel::Info => ("INFO", "\x1B[33;94m"),
        LogLevel::ClientError => ("CLNT", "\x1B[33;93m"),
        LogLevel::ServerError => ("SERV", "\x1B[33;91m"),
    };
    println!("{}[{}@{:?}]\x1B[33;0m {}", color, id, Local::now(), msg);
}

pub fn run(config: Arguments) {
    log(
        &format!("Starting server on port {}", config.port),
        LogLevel::Info,
    );
    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.port)).unwrap();
    let pool = ThreadPool::new(config.threads);

    let config = Arc::new(config);

    for stream in listener.incoming() {
        let config = Arc::clone(&config);
        match stream {
            Ok(stream) => {
                pool.execute(move || {
                    handle_connection(stream, config);
                });
            }
            Err(err) => {
                log(
                    &format!(" Client failed to connect: {}", err),
                    LogLevel::ClientError,
                );
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, config: Arc<Arguments>) {
    // Read max 1KB of request data
    let mut buffer = [0; 1024];

    if stream.read(&mut buffer).is_err() {
        log(
            &format!("Client sent malformed stream"),
            LogLevel::ClientError,
        );
        empty_response(&mut stream, HttpStatus::BadRequest);
        return;
    }

    let serve_path_str = &config.directory;

    let buffer_str = String::from_utf8_lossy(&buffer);
    let request = match Request::parse(&buffer_str) {
        Some(x) => x,
        None => {
            empty_response(&mut stream, HttpStatus::BadRequest);
            return;
        }
    };
    let client_address = match stream.local_addr() {
        Ok(addr) => addr.ip().to_string(),
        Err(_) => "Unknown".to_string(),
    };

    let user_agent = request.headers.get("User-Agent").unwrap_or(&"Unknown");

    log(
        &format!(
            "Request received:\naddress: {}\nuser-agent: {}\nmethod: {}\npath: {}\nprotocol: {}",
            client_address, user_agent, request.method, request.path, request.protocol
        ),
        LogLevel::Info,
    );

    let accepts_gzip = request
        .headers
        .get("Accept-Encoding")
        .and_then(|encodings| {
            for substr in encodings.split(',') {
                if substr.trim_start().starts_with("gzip") {
                    return Some(());
                }
            }
            None
        })
        .is_some();

    if request.protocol != "HTTP/1.1" {
        empty_response(&mut stream, HttpStatus::UnsupportedVersion);
        return;
    }

    let send_body = if request.method != "HEAD" {
        true
    } else {
        false
    };

    if request.method != "GET" && request.method != "HEAD" {
        empty_response(&mut stream, HttpStatus::NotAllowed);
        return;
    }

    match HttpContent::new(serve_path_str, request.path) {
        Some(content) => match content.get_bytes() {
            Ok(bytes) => success(
                &mut stream,
                &bytes,
                content.content_headers(),
                accepts_gzip,
                send_body,
            ),
            Err(_) => server_error(&mut stream, send_body),
        },
        None => match HttpContent::new(serve_path_str, "404.html") {
            Some(content) => match content.get_bytes() {
                Ok(bytes) => not_found(
                    &mut stream,
                    &bytes,
                    content.content_headers(),
                    accepts_gzip,
                    send_body,
                ),
                Err(_) => server_error(&mut stream, send_body),
            },
            None => server_error(&mut stream, send_body),
        },
    }
}

fn server_error(stream: &mut TcpStream, include_body: bool) {
    let content_headers = ContentHeaders {
        content_type: "text/plain",
        cache_age: 0,
        compress: false,
    };
    respond(
        stream,
        b"500 Internal Server Error",
        Some(content_headers),
        HttpStatus::ServerError,
        include_body,
    );
}

fn empty_response(stream: &mut TcpStream, status: HttpStatus) {
    respond(stream, b"", None, status, false);
}

fn success(
    stream: &mut TcpStream,
    bytebuffer: &[u8],
    content_headers: ContentHeaders,
    allow_compression: bool,
    include_body: bool,
) {
    let mut headers = ContentHeaders { ..content_headers };
    headers.compress = content_headers.compress && allow_compression;
    respond(
        stream,
        bytebuffer,
        Some(headers),
        HttpStatus::Ok,
        include_body,
    );
}

fn not_found(
    stream: &mut TcpStream,
    bytebuffer: &[u8],
    content_headers: ContentHeaders,
    allow_compression: bool,
    include_body: bool,
) {
    let mut headers = ContentHeaders { ..content_headers };
    headers.compress = content_headers.compress && allow_compression;
    respond(
        stream,
        bytebuffer,
        Some(headers),
        HttpStatus::NotFound,
        include_body,
    );
}

fn respond(
    stream: &mut TcpStream,
    bytebuffer: &[u8],
    content_headers: Option<ContentHeaders>,
    status: HttpStatus,
    include_body: bool,
) {
    let bytes = bytebuffer.to_vec();
    let mut response = Response::new(status, bytes);

    response.set_default_headers();
    if let Some(headers) = content_headers {
        response.set_content_headers(&headers);
        if headers.compress && response.compress_gzip().is_err() {
            log("Could not compress file", LogLevel::ServerError);
        }
    }

    let result = response.send(stream, include_body);

    if let Err(e) = result {
        log(
            &format!("Could not send a response: {}", e),
            LogLevel::ServerError,
        )
    }
}
