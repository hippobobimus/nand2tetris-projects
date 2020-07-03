use std::io::{BufReader, BufRead};
use std::fs::File;
use std::path::Path;
use regex::{Regex, RegexSet};
use crate::error::{Error, ErrorKind, Result};

#[derive(Debug, PartialEq)]
pub enum Command {
    ACommand(String),
    CCommand(String),
    LCommand(String),
}

#[derive(Debug)]
pub struct Parser {
    reader: std::io::BufReader<File>,
    pub raw_line: String,
    pub command: Option<Command>,
}

impl Parser {
    /// Creates a new Parser instance from an input filename.
    /// 
    pub fn new(filename: &Path) -> Result<Parser> {
        let file = File::open(filename)?;

        Ok(Parser {
            reader: BufReader::new(file),
            raw_line: String::new(),
            command: None,
        })
    }

    /// Reads the next line from the source file and extracts a command string
    /// if present.  The 'command' option field of the Parser instance is updated
    /// accordingly.
    /// 
    /// Returns a result containing the number of bytes present in the original
    /// line in the source file.
    ///
    /// Ok(0) will be returned when EOF is reached.
    ///
    pub fn advance(&mut self) -> Result<usize> {
        self.raw_line.clear();

        let bytes = self.reader.read_line(&mut self.raw_line)?;

        self.set_command()?;

        Ok(bytes)
    }

    /// Takes the currently loaded raw line from the source file, strips it of
    /// any comments and trims any remaining leading or trailing whitespace.
    ///
    /// The 'command' field of the Parser instance is then set to an Option
    /// containing the resultant String, or None if the String is empty.
    /// Subsequently, the 'command_type' is also set.
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

    /// Returns an option containing the type of the current command
    /// (A, C or L Command).  If there is no command present in the parser
    /// then 'None' will be returned.
    ///
    fn set_command_type(&mut self, cmd: &str) -> Result<usize> {
        let re_a = Regex::new(r"^@").unwrap();
        let re_c = RegexSet::new(&[
            r"^[[:alpha:]]+=[[:alpha:]01\-!+&|]+$",  // dest=comp
            r"^[[:alpha:]01\-!+&|]+;[[:alpha:]]+$",  // comp;jump
            r"^[[:alpha:]]+=[[:alpha:]01\-!+&|]+;[[:alpha:]]+$",  // dest=comp;jump
        ]).unwrap();
        let re_l = Regex::new(r"^\([[:word:]]+\)$").unwrap();

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

    ///
    ///
    ///
    pub fn symbol(&self) -> Result<String> {
        let (symbol, re) = match self.command {
            Some(Command::ACommand(ref cmd)) => {
                (cmd.clone(), Regex::new(r"^@").unwrap())
            },
            Some(Command::LCommand(ref cmd)) => {
                (cmd.clone(), Regex::new(r"^\(").unwrap())
            },
            _ => return Err(Error::new(ErrorKind::InvalidCmdType)),
        };

        let symbol = String::from(re.replace(&symbol[..], ""));

        Ok(symbol)
    }

    ///
    ///
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

    ///
    ///
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

    ///
    ///
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
    fn command_assignment() {
        let mut parser = temp_parser("\
            @VAR_1          // Example A-command with symbol.\n\
            @12             // Example A-command without symbol.\n\
            AMD=D|A         // Example C-command dest=comp\n\
            D&A;JNE         // Example C-command comp;jump\n\
            A=!D;null       // Example C-command dest=comp;jump\n\
            (LOOP_1)        // Example L-command.\
            ");

        let commands = vec![
            Command::ACommand(String::from("@VAR_1")),
            Command::ACommand(String::from("@12")),
            Command::CCommand(String::from("AMD=D|A")),
            Command::CCommand(String::from("D&A;JNE")),
            Command::CCommand(String::from("A=!D;null")),
            Command::LCommand(String::from("(LOOP_1)")),
        ];

        for cmd in commands {
            parser.advance().unwrap();
            assert_eq!(
                parser.command.take().unwrap(),
                cmd
            );
        }
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
}
