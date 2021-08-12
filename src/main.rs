use http_server::Arguments;
use std::env;
use std::option::Option;
use std::process;

fn collect_args(args: &Vec<String>) -> Option<Arguments> {
    let directory = args.get(1)?.to_string();
    let port: u32 = args.get(2).unwrap_or(&String::from("80")).parse().ok()?;
    Some(Arguments { directory, port })
}

fn main() {
    let args = env::args().collect();

    let config = collect_args(&args).unwrap_or_else(|| {
        println!("Usage: {} directory [port-number]", args[0]);
        process::exit(1);
    });

    http_server::run(config);
}
