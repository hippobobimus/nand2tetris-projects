use std::io::{BufReader, BufRead};
use std::fs::File;
use std::path::Path;
use regex::{Regex, RegexSet};
//use crate::code_translator;
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
    pub fn new(filename: &Path) -> Result<Parser> {  // &String
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
        let re_l = Regex::new(r"^\([[:alnum:]_]+\)$").unwrap();

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
                 //RegexSet::new(&[
                 Regex::new(r"(?x)
                    (^
                    (?P<dest>[[:alpha:]]+)  # dest=comp
                    =
                    (?P<comp>[[:alpha:]01\-!+&|]+)
                    )
                 ").unwrap()
                     //r"^[[:alpha:]]+=[[:alpha:]01\-!+&|]+$",  // dest=comp
                     //r"^[[:alpha:]01\-!+&|]+;[[:alpha:]]+$",  // comp;jump
                     //r"^[[:alpha:]]+=[[:alpha:]01\-!+&|]+;[[:alpha:]]+$",  // dest=comp;jump
                 //]).unwrap()
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn temp_parser() -> Parser {
        let text = "\
            @VAR_1          // Example A-command with symbol.\n\
            @12             // Example A-command without symbol.\n\
            AMD=D|A         // Example C-command dest=comp\n\
            D&A;JNE         // Example C-command comp;jump\n\
            A=!D;null       // Example C-command dest=comp;jump\n\
            (LOOP_1)        // Example L-command.\
            ";

        let mut file = NamedTempFile::new().unwrap();

        file.write_all(text.as_bytes()).unwrap();

        let parser = Parser::new(file.path()).unwrap();

        parser
    }

    #[test]
    fn command_type_assignment() {
        let mut parser = temp_parser();

        let commands = vec![
            Command::ACommand,
            Command::ACommand,
            Command::CCommand,
            Command::CCommand,
            Command::CCommand,
            Command::LCommand,
        ];

        for cmd in commands {
            parser.advance().unwrap();
            assert_eq!(
                parser.command_type().unwrap(),
                cmd
            );
        }
    }
}

// COMMANDS
//
//pub trait Command: std::fmt::Debug {
//    fn translate(&self) -> Result<u16>;
//}
//
//#[derive(Debug, PartialEq)]
//enum ALContent {
//    DecimalValue(u16),
//    Symbol(String),
//}
//
///// A-instruction
/////
//#[derive(Debug, PartialEq)]
//pub struct ACommand {
//    content: ALContent,
//}
//
//impl Command for ACommand {
//    fn translate(&self) -> Result<u16> {
//        match self.content {
//            ALContent::DecimalValue(x) => {
//                let y = x;
//                return Ok(y);
//            },
//            ALContent::Symbol(_) => Ok(0),
//        }
//    }
//}
//
//impl ACommand {
//    fn new(raw_content: &str) -> Result<ACommand> {
//        let content = raw_content.trim_matches('@');
//
//        if content.is_empty() {
//            return Err(Error::new(ErrorKind::InvalidSyntax));
//        };
//
//        match content.parse::<u16>() {
//            Ok(n) => {
//                return Ok(ACommand { content: ALContent::DecimalValue(n) });
//            },
//            Err(_) => {
//                // 'kind' API only available in nightly
//                // extend later
//                return Ok(ACommand {
//                        content: ALContent::Symbol(String::from(content))
//                });
//            },
//        }
//    }
//}
//
///// L-instruction
/////
//#[derive(Debug, PartialEq)]
//pub struct LCommand {
//    content: ALContent,
//}
//
//impl Command for LCommand {
//    fn translate(&self) -> Result<u16> {
//        println!("to do...");
//        Ok(0)
//    }
//}
//
//impl LCommand {
//    fn new(raw_content: &str) -> Result<LCommand> {
//        let chars_to_trim: &[char] = &['(', ')'];
//        
//        let content = ALContent::Symbol(String::from(
//                raw_content.trim_matches(chars_to_trim)));
//
//        Ok(LCommand { content })
//    }
//}
//
///// C-instruction
/////
//#[derive(Debug, PartialEq)]
//pub struct CCommand {
//    dest: Option<String>,
//    comp: Option<String>,
//    jump: Option<String>,
//}
//
//impl Command for CCommand {
//    fn translate(&self) -> Result<u16> {
//        let mut x = 0b1110_0000_0000_0000;
//        
//        let d = match self.dest {
//            Some(ref s) => code_translator::dest(&s[..])?,
//            None => 0b0000_0000_0000_0000,
//        };
//
//        let c = match self.comp {
//            Some(ref s) => code_translator::comp(&s[..])?,
//            None => 0b0000_0000_0000_0000,
//        };
//
//        let j = match self.jump {
//            Some(ref s) => code_translator::jump(&s[..])?,
//            None => 0b0000_0000_0000_0000,
//        };
//
//        x = x + d + c + j;
//
//        Ok(x)
//    }
//}
//
//impl CCommand {
//    fn new(content: &str) -> Result<CCommand> {
//        let contains_equals = content.contains('=');
//        let contains_semicolon = content.contains(';');
//
//        if contains_equals && contains_semicolon {
//            // dest=comp;jump
//            let mut cmd_iter = content.split(|c| c == '=' || c == ';')
//                .map(|x| String::from(x));
//            let dest = cmd_iter.next();
//            let comp = cmd_iter.next();
//            let jump = cmd_iter.next();
//
//            return Ok(CCommand {dest, comp, jump });
//
//        } else if contains_equals {
//            // dest=comp
//            let mut cmd_iter = content.split('=')
//                .map(|x| String::from(x));
//            let dest = cmd_iter.next();
//            let comp = cmd_iter.next();
//
//            return Ok(CCommand { dest, comp, jump: None });
//
//        } else if contains_semicolon {
//            // comp;jump
//            let mut cmd_iter = content.split(';')
//                .map(|x| String::from(x));
//            let comp = cmd_iter.next();
//            let jump = cmd_iter.next();
//
//            return Ok(CCommand {dest: None, comp, jump });
//
//        } else {
//            return Err(Error::new(ErrorKind::InvalidSyntax));
//        }
//    }
//}

//#[cfg(test)]
//mod tests {
//    use super::{*, ALContent::*};
//
//    #[test]
//    fn general_command_initialiser() {
//
//    }
//
//    #[test]
//    fn a_command_initialiser() {
//        let a_cmd_decimal = ACommand::new("@18").unwrap();
//        let a_cmd_symbol = ACommand::new("@R1").unwrap();
//        
//        assert_eq!(a_cmd_decimal, ACommand {
//            content: DecimalValue(18)
//        });
//        assert_eq!(a_cmd_symbol, ACommand {
//            content: Symbol(String::from("R1"))
//        });
//    }
//
//    #[test]
//    fn l_command_initialiser() {
//        let l_cmd_decimal = LCommand::new("(9)").unwrap();
//        let l_cmd_symbol = LCommand::new("(LOOP)").unwrap();
//        
//        assert_eq!(l_cmd_decimal, LCommand {
//            content: Symbol(String::from("9"))
//        });
//        assert_eq!(l_cmd_symbol, LCommand {
//            content: Symbol(String::from("LOOP"))
//        });
//    }
//
//    #[test]
//    fn c_command_initialiser() {
//        let c_cmd_1 = CCommand::new("DEST=COMP;JUMP").unwrap();
//        let c_cmd_2 = CCommand::new("COMP;JUMP").unwrap();
//        let c_cmd_3 = CCommand::new("DEST=COMP").unwrap();
//
//        assert_eq!(c_cmd_1, CCommand {
//            dest: Some(String::from("DEST")),
//            comp: Some(String::from("COMP")),
//            jump: Some(String::from("JUMP"))
//        }); 
//        assert_eq!(c_cmd_2, CCommand {
//            dest: None,
//            comp: Some(String::from("COMP")),
//            jump: Some(String::from("JUMP"))
//        }); 
//        assert_eq!(c_cmd_3, CCommand {
//            dest: Some(String::from("DEST")),
//            comp: Some(String::from("COMP")),
//            jump: None
//        }); 
//    }
//}
