use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use log::{self, Level, log_enabled};
use crate::code_translator;
use crate::config::Config;
use crate::error::Result;
use crate::parser::{Command, Parser};
use crate::symbols::SymbolTable;

/// Makes two passes through the input file.  First the symbol table is populated with entries
/// from L-pseudocommands.  In the second pass, A- and C-commands are translated into binary
/// machine instructions and written out to an output file.
///
/// Any symbolic A-commands encountered during the second pass are looked up in the symbol table
/// and added if not already present.
///
/// Returns Ok(()) if execution completes without error.
pub fn run(config: Config) -> Result<()> {
    let path = Path::new(&config.infile);

    let mut parser = Parser::new(path)?;

    log::debug!("Parser initialised from input file path\n{:#?}", parser);

    let output_file = File::create(config.outfile).unwrap();
    let mut output_writer = BufWriter::new(&output_file);

    let mut sym_table = SymbolTable::new();

    log::debug!("Symbol table before 1st pass\n{:#?}", sym_table);

    first_pass(&mut parser, &mut sym_table)?;

    // Reset parser.
    let mut parser = Parser::new(path)?;

    log::debug!("Symbol table between 1st and 2nd pass\n{:#?}", sym_table);

    second_pass(&mut parser, &mut sym_table, &mut output_writer)?;

    log::debug!("Symbol table after 2nd pass\n{:#?}", sym_table);

    output_writer.flush()?;

    Ok(())
}

/// Takes a Parser object and advances line-by-line through the input file buffered within it.
///
/// Each line is processed for present L-pseudocommands,
///
/// Returns Ok(0) if execution completes without error.
///
fn first_pass(parser: &mut Parser, sym_table: &mut SymbolTable) -> Result<u8> {
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
                        parser.raw_line.trim(),
                        parser.command,
                    );
                }

                process_l_cmd(parser, sym_table)?;
            },
        }
    }

    Ok(0)
}

/// Adds a new symbol to the symbol table with the current ROM address upon finding an L-Command.
/// Increments the ROM address when an A- or C-command is found, or does nothing if no command is
/// present.
///
/// Returns Ok(0) if execution completes without error.
///
fn process_l_cmd(parser: &mut Parser, sym_table: &mut SymbolTable) -> Result<u8> {
    match parser.command {
        Some(Command::LCommand(_)) => {
            let symbol = parser.symbol().unwrap();

            log::debug!("\
                L-Command. Adding entry to symbol table.\n\
                Symbol: {:#?}\n\
                ROM address: {:#?}\
                ",
                symbol,
                parser.get_rom_addr(),
            );

            sym_table.add_entry(symbol, parser.get_rom_addr())?;
        },
        Some(_) => {
            parser.inc_rom_addr();

            log::debug!("\
                Not an L-commmand (A or C-Command). Increment ROM address.\n\
                Updated ROM address: {}\
                ",
                parser.get_rom_addr(),
            );
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
fn second_pass<W>(parser: &mut Parser, sym_table: &mut SymbolTable, output_writer: &mut W) -> Result<u8>
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
                        parser.raw_line.trim(),
                        parser.command,
                    );
                }

                let line = match translate_line(parser, sym_table)? {
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
fn translate_line(parser: &mut Parser, sym_table: &mut SymbolTable) -> Result<Option<u16>> {
    let instruction = match parser.command {
        Some(Command::ACommand(_)) => {
            translate_a_cmd(parser, sym_table)?
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
fn translate_a_cmd(parser: &mut Parser, sym_table: &mut SymbolTable) -> Result<u16> {
    if log_enabled!(Level::Debug) {
        log::debug!("\
            A-Command\n\
            SYMBOL: {:?}\
            ",
            parser.symbol(),
        );
    }

    let symbol = parser.symbol()?;

    // Check for u16 or a symbol that needs to be added/looked up.
    match symbol.parse::<u16>() {
        Ok(b) => return Ok(b),
        Err(_) => {
            match sym_table.get_address(&symbol) {
                Some(b) => return Ok(b),
                None => {
                    let b = sym_table.add_entry(symbol, parser.get_ram_addr())?;
                    parser.inc_ram_addr()?;
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
