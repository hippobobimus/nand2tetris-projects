use std::env;
use std::process;
use assembler::config::Config;
//use assembler::parser::Parser;

fn main() {
    // let args: Vec<String> = env::args().collect();

    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("problem parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = assembler::runner::run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }

//    println!("{:?}", config);
//
//    let mut parser = Parser::initialise(&config.infile).unwrap();
//
//    println!("{:?}", parser);
//
//    for _ in 0..50 {
//        match parser.advance() {
//            Ok(0) => break,
//            Ok(_) => (),
//            Err(e) => panic!(e),
//        };
//
//        match &parser.cmd_buffer {
//            Some(cmd) => {
//            println!("{:?}\n", cmd);
//            },
//            None => (),
//        };

//        if parser.cmd_buffer == None {
//            break
//        };
//    }
}
