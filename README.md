Lightweight static web server written in Rust. Based on and expanded from the web server tutorial in [The Rust Programming Language](https://doc.rust-lang.org/book/ch20-00-final-project-a-web-server.html) book.

## Features

- Serves a directory of static files
- Supports gzip compression
- Supports GET and HEAD requests

## Building

Build:

`cargo build --release`

The built binary can be found at target/release/http-server(.exe)

## Running

`cargo run -- public`

Serves the 'public' directory included in repo
