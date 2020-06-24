use crate::config::Config;
use crate::parser::Parser;
use crate::error::Result;

pub fn run(config: Config) -> Result<()> {
    println!("{:?}", config);

    let mut parser = Parser::initialise(&config.infile)?;

    println!("{:?}", parser);

    for _ in 0..50 {
        match parser.advance() {
            Ok(0) => break,
            Ok(_) => (),
            Err(e) => panic!(e),
        };

        match &parser.cmd_buffer {
            Some(cmd) => {
            println!("{:?}", cmd);
            println!("Binary: {:#018b}\n", cmd.translate().unwrap());
            },
            None => (),
        };
    }

    Ok(())
}
