use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use log::{self, Level, log_enabled};
use crate::code_translator;
use crate::config::Config;
use crate::error::Result;
use crate::parser::{Command, Parser};

/// Makes two passes through the input file.  First the symbol table is populated with entries
/// from L-pseudocommands.  In the second pass, A- and C-commands are translated into binary
/// machine instructions and written out to an output file.
///
/// Any symbolic A-commands encountered during the second pass are looked up in the symbol table
/// and added if not already present.
///
/// Returns Ok(()) if execution completes without error.
///
pub fn run(config: Config) -> Result<()> {
    let path = Path::new(&config.infile);
    let mut parser = Parser::new(path)?;

    log::debug!("Parser initialised from input file path\n{:#?}", parser);

    let output_file = File::create(config.outfile).unwrap();
    let mut output_writer = BufWriter::new(&output_file);

    first_pass(&mut parser)?;

    parser.reset();

    log::debug!("Parser reset.\n{:#?}", parser);

    second_pass(&mut parser, &mut output_writer)?;

    log::debug!("Parser after both passes completed\n{:#?}", parser);

    output_writer.flush()?;

    Ok(())
}

/// Takes a Parser object and advances line-by-line through the input file buffered within it.
///
/// Each line is processed for present L-pseudocommands,
///
/// Returns Ok(0) if execution completes without error.
///
fn first_pass(parser: &mut Parser) -> Result<u8> {
    loop {
        match parser.advance()? {
            0 => {
                log::debug!("--- END OF FILE (1ST PASS) ---");
                break;
            },
            _ => {
                if log_enabled!(Level::Debug) {
                    log::debug!("\
                        >>> First pass: Parser advanced to next line. <<<\n\
                        RAW LINE READ: {:?}\n\
                        COMMAND: {:?}\
                        ",
                        parser.get_raw_line().trim(),
                        parser.get_command(),
                    );
                }

                process_l_cmd(parser)?;
            },
        }
    }

    Ok(0)
}

/// Adds a new label symbol to the symbol table with the current ROM address upon finding an
/// L-pseudocommand.  Increments the ROM address when an A- or C-command is found, or does nothing
/// if no command is present.
///
/// Returns Ok(0) if execution completes without error.
///
fn process_l_cmd(parser: &mut Parser) -> Result<u8> {
    match parser.get_command() {
        Some(Command::LCommand(_)) => {
            let symbol = parser.symbol().unwrap();

            log::debug!("\
                L-Command. Adding label to symbol table.\n\
                Symbol: {:#?}\n\
                ",
                symbol,
            );

            parser.insert_label(&symbol[..])?;
        },
        Some(_) => {
            parser.inc_rom_address();

            log::debug!("Not an L-commmand (A or C-Command). Increment ROM address.");
        },
        None => {
            log::debug!("Not a command. Continue to next line.");
        },
    };

    Ok(0)
}

/// Takes a Parser object and advances line-by-line through the input file buffered within it.
///
/// An attempt is made to translate each line into a binary machine instruction,  If successful
/// the instruction is written to the output writer.
///
/// Returns Ok(0) if execution completes without error.
///
fn second_pass<W>(parser: &mut Parser, output_writer: &mut W) -> Result<u8>
    where W: Write
{
    loop {
        match parser.advance()? {
            0 => {
                log::debug!("--- END OF FILE (2ND PASS) ---");
                break;
            },
            _ => {
                if log_enabled!(Level::Debug) {
                    log::debug!("\
                        >>> Second pass: Parser advanced to next line. <<<\n\
                        RAW LINE READ: {:?}\n\
                        COMMAND: {:?}\
                        ",
                        parser.get_raw_line().trim(),
                        parser.get_command(),
                    );
                }

                let line = match translate_line(parser)? {
                    Some(b) => b,
                    None => continue,
                };

                log::debug!("\
                    Translated assembly instruction into binary machine instruction\n\
                    MACHINE INSTRUCTION: {:016b}\
                    ", line);

                writeln!(output_writer, "{:016b}", line)?;
            },
        };
    }

    Ok(0)
}

/// Takes the current command and, if it is an A- or C-command, translates it into a binary machine
/// instruction.
///
/// Returns a result with an option that contains the instruction, or None if an A- or C-command
/// was not present.
///
fn translate_line(parser: &mut Parser) -> Result<Option<u16>> {
    let instruction = match parser.get_command() {
        Some(Command::ACommand(_)) => {
            translate_a_cmd(parser)?
        },
        Some(Command::CCommand(_)) => {
            translate_c_cmd(parser)?
        },
        _ => {
            log::debug!("Not an A- or C-command. Ignore and continue to next line.");
            return Ok(None);
        },
    };

    Ok(Some(instruction))
}

/// Translates an A-command into a binary machine instruction.
///
/// The supplied SymbolTable is referenced or updated as needed when symbolic A-Commands are found.
///
/// Returns a result containing the 16-bit machine instruction.
///
fn translate_a_cmd(parser: &mut Parser) -> Result<u16> {
    if log_enabled!(Level::Debug) {
        log::debug!("\
            A-Command\n\
            SYMBOL: {:?}\
            ",
            parser.symbol(),
        );
    }

    let symbol = parser.symbol()?;

    // Check for u16, a label/variable symbol that needs to be looked up, or a variable symbol
    // that needs to be added.
    match symbol.parse::<u16>() {
        Ok(b) => return Ok(b),
        Err(_) => {
            match parser.get_symbol_address(&symbol) {
                Some(b) => return Ok(b),
                None => {
                    log::debug!("New variable. Adding to symbol table.");

                    let b = parser.insert_variable(&symbol[..])?;

                    parser.inc_ram_address()?;

                    return Ok(b);
                },
            }
        },
    }
}

/// Translates a C-command into a binary machine instruction.
///
/// Returns a result containing the 16-bit machine instruction.
///
fn translate_c_cmd(parser: &mut Parser) -> Result<u16> {
    if log_enabled!(Level::Debug) {
        log::debug!("\
            C-Command\n\
            DEST: {:?}\n\
            COMP: {:?}\n\
            JUMP: {:?}\
            ",
            parser.dest(),
            parser.comp(),
            parser.jump(),
        );
    }

    let mut instruction = 0b1110_0000_0000_0000;
    
    let dest = match parser.dest()? {
        Some(ref s) => code_translator::dest(&s[..])?,
        None => 0b0000_0000_0000_0000,
    };

    let comp = match parser.comp()? {
        Some(ref s) => code_translator::comp(&s[..])?,
        None => 0b0000_0000_0000_0000,
    };

    let jump = match parser.jump()? {
        Some(ref s) => code_translator::jump(&s[..])?,
        None => 0b0000_0000_0000_0000,
    };

    instruction += dest + comp + jump;

    return Ok(instruction);
}

#[cfg(test)]
mod tests {
    use super::*;
    //use std::io::Write;
    use tempfile::NamedTempFile;

    fn temp_parser(text: &str) -> Parser {
        let mut file = NamedTempFile::new().unwrap();

        file.write_all(text.as_bytes()).unwrap();

        let parser = Parser::new(file.path()).unwrap();

        parser
    }

    #[test]
    fn try_first_pass() {
        let mut parser = temp_parser("\
            @VAR_1          // Example A-command with variable symbol.\n\
            @12             // Example A-command without symbol.\n\
            @LOOP_1         // Example A-command with label symbol.\n\
            AMD=D|A         // Example C-command dest=comp\n\
            D&A;JNE         // Example C-command comp;jump\n\
            A=!D;null       // Example C-command dest=comp;jump\n\
            (LOOP_1)        // Example L-command.\
            ");

        first_pass(&mut parser).unwrap();

        // L-pseudocommand with ROM address 6.
        assert_eq!(
            parser.get_symbol_address("LOOP_1").unwrap(),
            6,
        );

        // Symbolic A-commands should not be added to the symbol table.
        assert_eq!(
            parser.get_symbol_address("VAR_1"),
            None,
        );

        // C-commands should not be added to the symbol table.
        assert_eq!(
            parser.get_symbol_address("D&A;JNE"),
            None,
        );
    }

    #[test]
    fn try_second_pass() {
        let mut parser = temp_parser("\
            @VAR_1          // Example A-command with variable symbol.\n\
            @12             // Example A-command without symbol.\n\
            @LOOP_1         // Example A-command with label symbol.\n\
            AMD=D|A         // Example C-command dest=comp\n\
            D&A;JNE         // Example C-command comp;jump\n\
            A=!D;null       // Example C-command dest=comp;jump\n\
            (LOOP_1)        // Example L-command.\
            ");

        // Mimic action of 'first_pass' function.
        for _ in 0..6 {
            parser.inc_rom_address();
        };
        parser.insert_label("LOOP_1").unwrap();

        let mut output_buf: Vec<u8> = Vec::new();

        second_pass(&mut parser, &mut output_buf).unwrap();

        let output = String::from_utf8_lossy(&output_buf);

        println!("{:?}", output);

        assert_eq!(
            String::from("\
                0000000000010000\n\
                0000000000001100\n\
                0000000000000110\n\
                1110010101111000\n\
                1110000000000101\n\
                1110001101100000\n\
                "),
            output,
        );
    }
}
