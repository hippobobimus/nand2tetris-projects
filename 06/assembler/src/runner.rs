use std::fs::File;
use std::io::Write;
use std::io::BufWriter;
use std::path::Path;
use crate::config::Config;
use crate::parser::Parser;
use crate::error::{ErrorKind, Result};

pub fn run(config: Config) -> Result<()> {
    println!("{:?}", config);

    let path = Path::new(&config.infile);

    let mut parser = Parser::new(path)?;

    println!("{:?}", parser);

    let output_file = File::create(config.outfile).unwrap();
    let mut output_writer = BufWriter::new(&output_file);

    loop {
        match parser.advance() {
            Ok(0) => break,  // EOF
            Ok(_) => {
                println!("----------");
                println!("LINE: {}", parser.raw_line.trim());
                println!("CMD: {:?}", parser.command);
                println!("TYPE: {:?}", parser.command_type());
                //let line = cmd.translate().unwrap();
                //writeln!(&mut output_writer, "{:016b}", cmd)?;
            },
            Err(e) => panic!(e),
        };

//        match &parser.cmd_buffer {
//            Some(cmd) => {
//            println!("{:?}", cmd);
//
//            let line = cmd.translate().unwrap();
//            writeln!(&mut output_writer, "{:016b}", line)?;
//            },
//            None => (),
//        };
    }
//    for _ in 0..50 {
//        match parser.advance() {
//            Ok(0) => break,
//            Ok(_) => (),
//            Err(e) => panic!(e),
//        };
//
//        match &parser.cmd_buffer {
//            Some(cmd) => {
//            println!("{:?}", cmd);
//
//            let line = cmd.translate().unwrap();
//            writeln!(&mut output_writer, "{:016b}", line)?;
//            },
//            None => (),
//        };
//    }

    output_writer.flush()?;


    Ok(())
}
