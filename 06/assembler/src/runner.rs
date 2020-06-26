use std::fs::File;
use std::io::Write;
use std::io::BufWriter;
use crate::config::Config;
use crate::parser::Parser;
use crate::error::Result;

pub fn run(config: Config) -> Result<()> {
    println!("{:?}", config);

    let mut parser = Parser::initialise(&config.infile)?;

    println!("{:?}", parser);

    let output_file = File::create(config.outfile).unwrap();
    let mut output_writer = BufWriter::new(&output_file);

    for _ in 0..50 {
        match parser.advance() {
            Ok(0) => break,
            Ok(_) => (),
            Err(e) => panic!(e),
        };

        match &parser.cmd_buffer {
            Some(cmd) => {
            println!("{:?}", cmd);

            let line = cmd.translate().unwrap();
            writeln!(&mut output_writer, "{:016b}", line)?;
            },
            None => (),
        };
    }

    output_writer.flush()?;


    Ok(())
}
