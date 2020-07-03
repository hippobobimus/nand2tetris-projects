use std::fs::File;
use std::io::Write;
use std::io::BufWriter;
use std::path::Path;
use crate::config::Config;
use crate::parser::{Command, Parser};
use crate::error::{Result};
use crate::code_translator;

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
                println!("SYMBOL: {:?}", parser.symbol());
                println!("DEST: {:?}", parser.dest());
                println!("COMP: {:?}", parser.comp());
                println!("JUMP: {:?}", parser.jump());
 
                let line = match parser.command {
                    Some(Command::ACommand(_)) => {
                        parser.symbol().unwrap().parse::<u16>()?
                        //println!("Binary: {:016b}", b);
                    },
                    Some(Command::CCommand(_)) => {
                        let b = 0b1110_0000_0000_0000;
                        
                        let d = match parser.dest()? {
                            Some(ref s) => code_translator::dest(&s[..])?,
                            None => 0b0000_0000_0000_0000,
                        };

                        let c = match parser.comp()? {
                            Some(ref s) => code_translator::comp(&s[..])?,
                            None => 0b0000_0000_0000_0000,
                        };

                        let j = match parser.jump()? {
                            Some(ref s) => code_translator::jump(&s[..])?,
                            None => 0b0000_0000_0000_0000,
                        };

                        b + d + c + j

                        //println!("Binary: {:016b}", b);
                    },
                    Some(Command::LCommand(_)) => {
                        1
                    },
                    None => continue,
                };

                println!("Binary: {:016b}", line);
                writeln!(&mut output_writer, "{:016b}", line)?;

            },
            Err(e) => panic!(e),
        };
    }

    output_writer.flush()?;


    Ok(())
}
                //println!("TYPE: {:?}", parser.command_type);
                //let line = cmd.translate().unwrap();
                //writeln!(&mut output_writer, "{:016b}", cmd)?;
//            },
//            Err(e) => panic!(e),
//        };

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
//
//    output_writer.flush()?;
//
//
//    Ok(())
//}
