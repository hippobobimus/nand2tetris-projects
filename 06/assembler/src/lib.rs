pub mod config {
    #[derive(Debug)]
    pub struct Config {
        pub infile: String,
    }

    impl Config {
        pub fn new(mut args: std::env::Args) -> Result<Config, &'static str> {
            args.next();

            let infile = match args.next() {
                Some(arg) => arg,
                None => return Err("An input filename was not received."),
            };

            Ok(Config { infile })
        }
    }
}

pub mod parser {
    use std::io::{self, BufReader, BufRead};
    use std::fs::File;
    use std::num::ParseIntError;
    use std::fmt;
    use std::error;

    type Result<T> = std::result::Result<T, ParserError>;

    #[derive(Debug)]
    pub enum ParserError {
        IO(io::Error),
        ParseInt(ParseIntError),
        Syntax(String),
    }

    impl fmt::Display for ParserError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                ParserError::IO(ref e) => e.fmt(f),
                ParserError::ParseInt(ref e) => e.fmt(f),
                ParserError::Syntax(ref e) =>
                    write!(f, "Syntax error in line: {}", e),
            }
        }
    }

    impl error::Error for ParserError {
        fn source(&self) -> Option<&(dyn error::Error + 'static)> {
            match *self {
                ParserError::IO(ref e) => Some(e),
                ParserError::ParseInt(ref e) => Some(e),
                ParserError::Syntax(_) => None,
            }
        }
    }

    impl From<io::Error> for ParserError {
        fn from(err: io::Error) -> ParserError {
            ParserError::IO(err)
        }
    }

    impl From<ParseIntError> for ParserError {
        fn from(err: ParseIntError) -> ParserError {
            ParserError::ParseInt(err)
        }
    }

    #[derive(Debug)]
    pub struct Parser {
        pub cmd_buffer: Option<Box<dyn Command>>,
        reader: std::io::BufReader<File>,
        end_of_file: bool,
    }
    
    impl Parser {
        /// Creates a new Parser instance.
        /// 
        pub fn initialise(filename: &String) -> Result<Parser> {
            let file = File::open(filename)?;

            Ok(Parser {
                reader: BufReader::new(file),
                cmd_buffer: None,
                end_of_file: false,
            })
        }

        /// Loads the next command from the source file into the Parser
        /// instance's 'cmd_buffer'.
        ///
        pub fn advance(&mut self) -> Result<usize> {
            if self.end_of_file == true {
                println!(">>> Can't advance, already reached end of file <<<");
                return Ok(0);
            }

            self.cmd_buffer = None;

            let mut raw_content = String::new();

            match self.reader.read_line(&mut raw_content) {
                Ok(0) => {
                    println!(">>> Reached end of file <<<");
                    self.end_of_file = true;
                    return Ok(0);
                },
                Ok(b) => {
                    println!("Read line composed of {} bytes.", b);
                    self.cmd_buffer = Parser::new_command(raw_content)?;
                    return Ok(b);
                },
                Err(e) => {
                    return Err(ParserError::IO(e));
                }
            }
        }

        fn new_command(raw_content: String) -> Result<Option<Box<dyn Command>>> {
            let content = Parser::strip_comments(raw_content);
            let content = content.trim();

            if content.is_empty() {
                println!("No commands found in line.\n");
                return Ok(None);
            }

            if content.starts_with("@") {
                match ACommand::new(content) {
                    Ok(cmd) => return Ok(Some(Box::new(cmd))),
                    Err(e) => return Err(e),
                }
            } else if content.starts_with("(") {
                match LCommand::new(content) {
                    Ok(cmd) => return Ok(Some(Box::new(cmd))),
                    Err(e) => return Err(e),
                }
            } else {
                match CCommand::new(content) {
                    Ok(cmd) => return Ok(Some(Box::new(cmd))),
                    Err(e) => return Err(e),
                }
            };
        }

        fn strip_comments(mut input: String) -> String {
            let comment_offset = input.find("//").unwrap_or(input.len());

            input.replace_range(comment_offset.., "");

            input
        }
        
    }

    // COMMANDS
    //
    pub trait Command: std::fmt::Debug {}
    pub trait SymbolicCommand: Command {}
    impl<T: SymbolicCommand> Command for T {}

    #[derive(Debug, PartialEq)]
    enum ALContent {
        DecimalValue(u8),
        Symbol(String),
    }

    #[derive(Debug, PartialEq)]
    pub struct ACommand {
        content: ALContent,
    }

    impl SymbolicCommand for ACommand {}

    impl ACommand {
        fn new(raw_content: &str) -> Result<ACommand> {
            let content = raw_content.trim_matches('@');

            match content.parse::<u8>() {
                Ok(n) => {
                    return Ok(ACommand { content: ALContent::DecimalValue(n) });
                },
                Err(_) => {
                    // 'kind' API only available in nightly
                    // extend later
                    return Ok(ACommand { content: ALContent::Symbol(
                                String::from(content)) });
                },
            }
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct LCommand {
        content: ALContent,
    }

    impl SymbolicCommand for LCommand {}

    impl LCommand {
        fn new(raw_content: &str) -> Result<LCommand> {
            let chars_to_trim: &[char] = &['(', ')'];
            
            let content = ALContent::Symbol(String::from(
                    raw_content.trim_matches(chars_to_trim)));

            Ok(LCommand { content })
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct CCommand {
        dest: Option<String>,
        comp: Option<String>,
        jump: Option<String>,
    }

    impl Command for CCommand {}

    impl CCommand {
        fn new(content: &str) -> Result<CCommand> {
            let contains_equals = content.contains('=');
            let contains_semicolon = content.contains(';');

            if contains_equals && contains_semicolon {
                // dest=comp;jump
                let mut cmd_iter = content.split(|c| c == '=' || c == ';')
                    .map(|x| String::from(x));
                let dest = cmd_iter.next();
                let comp = cmd_iter.next();
                let jump = cmd_iter.next();

                return Ok(CCommand {dest, comp, jump });

            } else if contains_equals {
                // dest=comp
                let mut cmd_iter = content.split('=')
                    .map(|x| String::from(x));
                let dest = cmd_iter.next();
                let comp = cmd_iter.next();

                return Ok(CCommand { dest, comp, jump: None });

            } else if contains_semicolon {
                // comp;jump
                let mut cmd_iter = content.split(';')
                    .map(|x| String::from(x));
                let comp = cmd_iter.next();
                let jump = cmd_iter.next();

                return Ok(CCommand {dest: None, comp, jump });

            } else {
                return Err(ParserError::Syntax(String::from(content)));
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{*, ALContent::*};

        #[test]
        fn general_command_initialiser() {

        }

        #[test]
        fn a_command_initialiser() {
            let a_cmd_decimal = ACommand::new("@18").unwrap();
            let a_cmd_symbol = ACommand::new("@R1").unwrap();
            
            assert_eq!(a_cmd_decimal, ACommand {
                content: DecimalValue(18)
            });
            assert_eq!(a_cmd_symbol, ACommand {
                content: Symbol(String::from("R1"))
            });
        }

        #[test]
        fn l_command_initialiser() {
            let l_cmd_decimal = LCommand::new("(9)").unwrap();
            let l_cmd_symbol = LCommand::new("(LOOP)").unwrap();
            
            assert_eq!(l_cmd_decimal, LCommand {
                content: Symbol(String::from("9"))
            });
            assert_eq!(l_cmd_symbol, LCommand {
                content: Symbol(String::from("LOOP"))
            });
        }

        #[test]
        fn c_command_initialiser() {
            let c_cmd_1 = CCommand::new("DEST=COMP;JUMP").unwrap();
            let c_cmd_2 = CCommand::new("COMP;JUMP").unwrap();
            let c_cmd_3 = CCommand::new("DEST=COMP").unwrap();

            assert_eq!(c_cmd_1, CCommand {
                dest: Some(String::from("DEST")),
                comp: Some(String::from("COMP")),
                jump: Some(String::from("JUMP"))
            }); 
            assert_eq!(c_cmd_2, CCommand {
                dest: None,
                comp: Some(String::from("COMP")),
                jump: Some(String::from("JUMP"))
            }); 
            assert_eq!(c_cmd_3, CCommand {
                dest: Some(String::from("DEST")),
                comp: Some(String::from("COMP")),
                jump: None
            }); 
        }
    }
}
