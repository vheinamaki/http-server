use chrono::Local;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

pub mod server;

use server::{ThreadPool, Request, Response, HttpContent, HttpStatus, ContentHeaders};

pub struct Arguments {
    pub directory: String,
    pub port: u32,
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
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.port)).unwrap();
    let pool = ThreadPool::new(4);

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
    let mut buffer = [0; 1024];

    if stream.read(&mut buffer).is_err() {
        log(
            &format!("Client sent malformed stream"),
            LogLevel::ClientError,
        );
        return;
    }

    let serve_path_str = &config.directory;

    let buffer_str = String::from_utf8_lossy(&buffer);
    let request = match Request::parse(&buffer_str) {
        Some(x) => x,
        None => {
            bad_request(&mut stream);
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

    if request.protocol != "HTTP/1.1" || request.method != "GET" {
        bad_request(&mut stream);
        return;
    }

    match HttpContent::new(serve_path_str, request.path) {
        Some(content) => match content.get_bytes() {
            Ok(bytes) => success(&mut stream, &bytes, content.content_headers()),
            Err(_) => server_error(&mut stream),
        },
        None => match HttpContent::new(serve_path_str, "404.html") {
            Some(content) => match content.get_bytes() {
                Ok(bytes) => not_found(
                    &mut stream,
                    &bytes,
                    content.content_headers()
                ),
                Err(_) => server_error(&mut stream),
            },
            None => server_error(&mut stream),
        },
    }
}

fn server_error(stream: &mut TcpStream) {
    let content_headers = ContentHeaders {
        content_type: "text/plain",
        cache_age: 0
    };
    respond(
        stream,
        b"500 Internal Server Error",
        Some(content_headers),
        HttpStatus::ServerError,
    );
}

fn bad_request(stream: &mut TcpStream) {
    respond(stream, b"", None, HttpStatus::BadRequest);
}

fn success(stream: &mut TcpStream, bytebuffer: &[u8], content_headers: ContentHeaders) {
    respond(stream, bytebuffer, Some(content_headers), HttpStatus::Ok);
}

fn not_found(stream: &mut TcpStream, bytebuffer: &[u8], content_headers: ContentHeaders) {
    respond(stream, bytebuffer, Some(content_headers), HttpStatus::NotFound);
}

fn respond(stream: &mut TcpStream, bytebuffer: &[u8], content_headers: Option<ContentHeaders>, status: HttpStatus) {
    let mut response = Response::new(status, bytebuffer);

    response.set_default_headers();
    if let Some(headers) = content_headers {
        response.set_content_headers(&headers);
    }

    let result = response.send(stream);

    if let Err(e) = result {
        log(
            &format!("Could not send a response: {}", e),
            LogLevel::ServerError,
        )
    }
}
