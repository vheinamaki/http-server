use clap::{value_t, App, Arg};
use http_server::Arguments;

fn main() {
    let args = App::new("http-server")
        .version("0.1.0")
        .arg(
            Arg::with_name("DIRECTORY")
                .help("The directory to serve. Should contain an index.html at minimum")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .help("Port to run the server on")
                .default_value("80"),
        )
        .arg(
            Arg::with_name("threads")
                .short("t")
                .long("threads")
                .help("Number of threads to allocate for request handling")
                .default_value("2"),
        )
        .get_matches();

    let config = Arguments {
        directory: String::from(args.value_of("DIRECTORY").unwrap()),
        port: value_t!(args.value_of("port"), u16).unwrap_or_else(|e| e.exit()),
        threads: value_t!(args.value_of("port"), usize).unwrap_or_else(|e| e.exit()),
    };
    http_server::run(config);
}
