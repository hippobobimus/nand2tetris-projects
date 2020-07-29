use std::io::{BufReader, BufRead, Seek, SeekFrom};
use std::fs::File;
use std::path::Path;
use regex::{Regex, RegexSet};
use crate::error::{Error, ErrorKind, Result};
use crate::symbols::SymbolTable;

/// Different types of Command; A- or C-instructions, or L-pseudocommands along with their String
/// representation.
///
#[derive(Debug, PartialEq)]
pub enum Command {
    ACommand(String),
    CCommand(String),
    LCommand(String),
}

/// A struct that encapsulates the current state of the parser.  It holds a BufReader for the input
/// file, as well as the last raw line read and any command contained within that line.  A
/// SymbolTable tracks variable and label symbols along with their allocated RAM/ROM addresses.
///
#[derive(Debug)]
pub struct Parser {
    reader: std::io::BufReader<File>,
    raw_line: String,
    command: Option<Command>,
    symbol_table: SymbolTable,
}

impl Parser {
    /// Takes a reference to the Path of an input file and returns a Result containing a new Parser
    /// instance.
    ///
    /// An error will be returned if opening the file identified by the given Path returns an
    /// error.
    /// 
    pub fn new(filename: &Path) -> Result<Parser> {
        let file = File::open(filename)?;

        Ok(Parser {
            reader: BufReader::new(file),
            raw_line: String::new(),
            command: None,
            symbol_table: SymbolTable::new(),
        })
    }

    /// Reads the next line and extracts a command string if present, updating the 'command' option
    /// field of the Parser instance appropriately.
    /// 
    /// Returns a result containing the number of bytes present in the original line read from the
    /// source file.
    ///
    /// Ok(0) will be returned when EOF is reached.
    ///
    pub fn advance(&mut self) -> Result<usize> {
        self.raw_line.clear();

        let bytes = self.reader.read_line(&mut self.raw_line)?;

        self.set_command()?;

        Ok(bytes)
    }

    /// Takes the currently loaded raw line from the source file, strips it of any comments and
    /// trims any remaining leading or trailing whitespace.
    ///
    /// If the remaining line content is not empty, then it is used to set the 'command' field of
    /// the Parser instance.  Otherwise the 'command' field is set to None.
    ///
    fn set_command(&mut self) -> Result<usize> {
        self.command = None;

        let mut cmd = self.raw_line.clone();

        let comment_offset = cmd.find("//").unwrap_or(cmd.len());

        cmd.replace_range(comment_offset.., "");
        let cmd = cmd.trim();
        
        if cmd.is_empty() {
            return Ok(0);
        } else {
            self.set_command_type(cmd)?;
        }
        Ok(0)
    }

    /// Takes an input &str and determines whether it is an A-, C- or L-command, setting the
    /// 'command' field fo the Parser instance appropriately.
    ///
    /// Returns Ok(0) upon successful execution.
    ///
    fn set_command_type(&mut self, cmd: &str) -> Result<usize> {
        let re_a = Regex::new(r"^@").unwrap();
        let re_c = RegexSet::new(&[
            r"^[[:alpha:]]+=[[:alpha:]01\-!+&|]+$",  // dest=comp
            r"^[[:alpha:]01\-!+&|]+;[[:alpha:]]+$",  // comp;jump
            r"^[[:alpha:]]+=[[:alpha:]01\-!+&|]+;[[:alpha:]]+$",  // dest=comp;jump
        ]).unwrap();
        let re_l = Regex::new(r"^\([[:word:].$]+\)$").unwrap();

        if re_a.is_match(cmd) {
            self.command = Some(Command::ACommand(String::from(cmd)));
        } else if re_c.is_match(cmd) {
            self.command = Some(Command::CCommand(String::from(cmd)));
        } else if re_l.is_match(cmd) {
            self.command = Some(Command::LCommand(String::from(cmd)));
        } else {
            return Err(Error::new(ErrorKind::InvalidSyntax));
        }

        Ok(0)
    }

    /// Returns a Result containing the variable or label symbol, or decimal string contained
    /// within the current command.
    ///
    /// This method can only be called on A- or L-commands and will otherwise return an error.
    ///
    pub fn symbol(&self) -> Result<String> {
        let (raw_symbol, re) = match self.command {
            Some(Command::ACommand(ref cmd)) => {
                (cmd.clone(), Regex::new(r"^@(?P<symbol>[[:word:].$]+)$").unwrap())
            },
            Some(Command::LCommand(ref cmd)) => {
                (cmd.clone(), Regex::new(r"^[\(](?P<symbol>[[:word:].$]+)[\)]$").unwrap())
            },
            _ => return Err(Error::new(ErrorKind::InvalidCmdType)),
        };

        let symbol = String::from(re.replace(&raw_symbol[..], "$symbol"));

        Ok(symbol)
    }

    /// Returns an Option containing the 'dest' component of the current C-command string, within
    /// an outer Result.
    ///
    /// This method can only be called on C-commands and will otherwise return an error.
    ///
    pub fn dest(&self) -> Result<Option<String>> {
        let (command, re) = match self.command {
            Some(Command::CCommand(ref cmd)) => {
                (cmd.clone(),
                 Regex::new(r"(^(?P<dest>[[:alpha:]]+)=)").unwrap()
                )
            },
            _ => return Err(Error::new(ErrorKind::InvalidCmdType)),
        };

        let caps = match re.captures(&command[..]) {
            Some(c) => c,
            None => return Ok(None),
        };

        let dest = String::from(caps.name("dest").unwrap().as_str());

        Ok(Some(dest))
    }

    /// Returns an Option containing the 'comp' component of the current C-command string, within
    /// an outer Result.
    ///
    /// This method can only be called on C-commands and will otherwise return an error.
    ///
    pub fn comp(&self) -> Result<Option<String>> {
        let (command, re) = match self.command {
            Some(Command::CCommand(ref cmd)) => {
                (cmd.clone(),
                 Regex::new(r"(?x)
                    (^(?P<comp_0>[[:alpha:]01\-!+&|]+);) |
                    (=(?P<comp_1>[[:alpha:]01\-!+&|]+);?)"
                 ).unwrap()
                )
            },
            _ => return Err(Error::new(ErrorKind::InvalidCmdType)),
        };

        let caps = match re.captures(&command[..]) {
            Some(c) => c,
            None => return Ok(None),
        };

        let comp = String::from(caps.name("comp_0").or_else(|| caps.name("comp_1")).unwrap().as_str());

        Ok(Some(comp))
    }

    /// Returns an Option containing the 'jump' component of the current C-command string, within
    /// an outer Result.
    ///
    /// This method can only be called on C-commands and will otherwise return an error.
    ///
    pub fn jump(&self) -> Result<Option<String>> {
        let (command, re) = match self.command {
            Some(Command::CCommand(ref cmd)) => {
                (cmd.clone(),
                 Regex::new(r"(;(?P<jump>[[:alpha:]]+)$)").unwrap()
                )
            },
            _ => return Err(Error::new(ErrorKind::InvalidCmdType)),
        };

        let caps = match re.captures(&command[..]) {
            Some(c) => c,
            None => return Ok(None),
        };

        let jump = String::from(caps.name("jump").unwrap().as_str());

        Ok(Some(jump))
    }

    /// Returns a reference to the last raw line from the input file read by the Parser.
    ///
    pub fn get_raw_line(&self) -> &String {
        &self.raw_line
    }

    /// Returns a reference to an Option containing the current Command loaded into the Parser.
    ///
    pub fn get_command(&self) -> &Option<Command> {
        &self.command
    }

    /// Increments the next available RAM address used when adding a new variable to the symbol
    /// table.
    ///
    pub fn inc_ram_address(&mut self) -> Result<u8> {
        self.symbol_table.inc_ram_address()
    }

    /// Increments the next available ROM address used when adding a new label to the symbol
    /// table.
    ///
    pub fn inc_rom_address(&mut self) {
        self.symbol_table.inc_rom_address();
    }

    /// Adds a new label to the symbol table and returns a Result containing the allocated ROM
    /// address.
    ///
    pub fn insert_label(&mut self, symbol: &str) -> Result<u16> {
        self.symbol_table.insert_label(symbol)
    }

    /// Adds a new variable to the symbol table and returns a Result containing the allocated RAM
    /// address.
    ///
    pub fn insert_variable(&mut self, symbol: &str) -> Result<u16> {
        self.symbol_table.insert_variable(symbol)
    }

    /// Takes a symbol &str and returns an Option containing the RAM/ROM address allocated to it.
    /// None is returned if the symbol is not present in the symbol table.
    ///
    pub fn get_symbol_address(&self, symbol: &str) -> Option<u16> {
        self.symbol_table.get_address(symbol)
    }

    /// Clears the current raw line and command loaded into the Parser instance and resets it back
    /// to reading from the beginning of the source file.
    ///
    pub fn reset(&mut self) {
        self.reader.get_mut().seek(SeekFrom::Start(0)).unwrap();
        self.raw_line.clear();
        self.command = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn temp_parser(text: &str) -> Parser {
        let mut file = NamedTempFile::new().unwrap();

        file.write_all(text.as_bytes()).unwrap();

        let parser = Parser::new(file.path()).unwrap();

        parser
    }

    #[test]
    fn command_assignment_and_eof() {
        let mut parser = temp_parser("\
            @VAR_1.$TEST    // Example A-command with variable symbol.\n\
            @12             // Example A-command without symbol.\n\
            AMD=D|A         // Example C-command dest=comp\n\
            D&A;JNE         // Example C-command comp;jump\n\
            A=!D;null       // Example C-command dest=comp;jump\n\
            ($TEST.LOOP_1)  // Example L-command with label symbol.\
            ");

        let commands = vec![
            Command::ACommand(String::from("@VAR_1.$TEST")),
            Command::ACommand(String::from("@12")),
            Command::CCommand(String::from("AMD=D|A")),
            Command::CCommand(String::from("D&A;JNE")),
            Command::CCommand(String::from("A=!D;null")),
            Command::LCommand(String::from("($TEST.LOOP_1)")),
        ];

        for cmd in commands {
            parser.advance().unwrap();
            assert_eq!(
                parser.command.take().unwrap(),
                cmd
            );
        }

        let eof = parser.advance().unwrap();

        assert_eq!(eof, 0);
    }

    #[test]
    #[should_panic(expected = "invalid syntax")]
    fn command_syntax_error() {
        let mut parser = temp_parser("\
            notacommand\n\
            ");

        parser.advance().unwrap();
    }

    #[test]
    fn retrieve_symbol() {
        let mut parser = temp_parser("\
            @VAR_1.$TEST    // Example A-command with variable symbol.\n\
            ($TEST.LOOP_1)  // Example L-command with label symbol.\n\
            ");

        let expected = vec![
            "VAR_1.$TEST",
            "$TEST.LOOP_1",
        ];

        for item in expected {
            parser.advance().unwrap();

            let symbol = parser.symbol().unwrap();

            assert_eq!(item, symbol);
        }
    }

    #[test]
    #[should_panic(expected = "this function cannot act on Commands of this type")]
    fn retrieve_symbol_from_c_cmd() {
        let mut parser = temp_parser("\
            AMD=D|A         // Example C-command dest=comp\n\
            ");

        parser.advance().unwrap();

        parser.symbol().unwrap();
    }

    #[test]
    #[should_panic(expected = "this function cannot act on Commands of this type")]
    fn retrieve_symbol_from_non_cmd() {
        let mut parser = temp_parser("\
            // Just a comment line, not a command.\n\
            ");

        parser.advance().unwrap();

        parser.symbol().unwrap();
    }

    #[test]
    fn dest_comp_jump_assignment() {
        let mut parser = temp_parser("\
            AMD=D|A         // Example C-command dest=comp\n\
            D&A;JNE         // Example C-command comp;jump\n\
            A=!D;null       // Example C-command dest=comp;jump\n\
            ");

        let commands = vec![
            ("AMD", "D|A", ""),
            ("", "D&A", "JNE"),
            ("A", "!D", "null"),
        ];

        for cmd in commands {
            parser.advance().unwrap();
            assert_eq!(
                String::from(cmd.0),
                parser.dest().unwrap().unwrap_or(String::from(""))
            );
            assert_eq!(
                String::from(cmd.1),
                parser.comp().unwrap().unwrap_or(String::from(""))
            );
            assert_eq!(
                String::from(cmd.2),
                parser.jump().unwrap().unwrap_or(String::from(""))
            );
        }
    }

    fn test_a_cmd() -> Parser {
        let mut parser = temp_parser("\
            @VAR_1.$TEST    // Example A-command with variable symbol.\n\
            ");

        parser.advance().unwrap();

        parser
    }

    fn test_l_cmd() -> Parser {
        let mut parser = temp_parser("\
            ($TEST.LOOP_1)  // Example L-command with label symbol.\n\
            ");

        parser.advance().unwrap();

        parser
    }

    #[test]
    #[should_panic(expected = "this function cannot act on Commands of this type")]
    fn dest_invalid_a_cmd() {
        test_a_cmd().dest().unwrap();
    }

    #[test]
    #[should_panic(expected = "this function cannot act on Commands of this type")]
    fn dest_invalid_l_cmd() {
        test_l_cmd().dest().unwrap();
    }

    #[test]
    #[should_panic(expected = "this function cannot act on Commands of this type")]
    fn comp_invalid_a_cmd() {
        test_a_cmd().comp().unwrap();
    }

    #[test]
    #[should_panic(expected = "this function cannot act on Commands of this type")]
    fn comp_invalid_l_cmd() {
        test_l_cmd().comp().unwrap();
    }

    #[test]
    #[should_panic(expected = "this function cannot act on Commands of this type")]
    fn jump_invalid_a_cmd() {
        test_a_cmd().jump().unwrap();
    }

    #[test]
    #[should_panic(expected = "this function cannot act on Commands of this type")]
    fn jump_invalid_l_cmd() {
        test_l_cmd().jump().unwrap();
    }
    #[test]
    fn check_reset() {
        let mut parser = temp_parser("\
            @VAR_1.$TEST    // Example A-command with variable symbol.\n\
            ($TEST.LOOP_1)  // Example L-command with label symbol.\n\
            ");
        
        let expected = vec![
            "@VAR_1.$TEST    // Example A-command with variable symbol.\n",
            "($TEST.LOOP_1)  // Example L-command with label symbol.\n",
        ];

        parser.advance().unwrap();
        assert_eq!(parser.get_raw_line(), expected[0]);

        parser.advance().unwrap();
        assert_eq!(parser.get_raw_line(), expected[1]);

        parser.reset();
        assert_eq!(parser.get_raw_line(), "");
        assert_eq!(*parser.get_command(), None);

        parser.advance().unwrap();
        assert_eq!(parser.get_raw_line(), expected[0]);
    }
}
