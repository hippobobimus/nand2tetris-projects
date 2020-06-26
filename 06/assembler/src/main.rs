use std::env;
use std::process;
use assembler::config::Config;

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("problem parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = assembler::runner::run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}
