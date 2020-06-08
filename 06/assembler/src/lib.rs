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
//    use std::error::Error;

    #[derive(Debug)]
    pub struct Parser {
        pub cmd_buffer: Option<Box<dyn Command>>,
        reader: std::io::BufReader<File>,
        end_of_file: bool,
    }
    
    impl Parser {
        /// Creates a new Parser instance.
        /// 
        pub fn initialise(filename: &String) -> Result<Parser, io::Error> {
            let file = File::open(filename)?;

            Ok(Parser {
                reader: BufReader::new(file),
                cmd_buffer: None, //String::new(),
                end_of_file: false,
            })
        }

        /// Loads the next command from the source file into the Parser
        /// instance's 'cmd_buffer'.
        ///
        pub fn advance(&mut self) -> Result<usize, io::Error> {
            if self.end_of_file == true {
                println!("Reached end of file");
                return Ok(0);
            }

            self.cmd_buffer = None;

            let mut raw_content = String::new();

            match self.reader.read_line(&mut raw_content) {
                Ok(0) => {
                    println!("Reached EOF!");
                    self.end_of_file = true;
                    return Ok(0);
                },
                Ok(b) => {
                    println!("Successfully read line with {} bytes.", b);
                    //let trimmed_content = content.trim();
                    self.cmd_buffer = Parser::new_command(raw_content);
                    return Ok(b);
                },
                Err(e) => {
                    return Err(e);
                }
            }
        }

        fn new_command(raw_content: String) -> Option<Box<dyn Command>> {
            let content = Parser::strip_comments(raw_content);
            let content = content.trim();

            if content.is_empty() {
                println!("Skipping line");
                return None;
            }

            if content.starts_with("@") {
                return Some(Box::new(ACommand::new(content).unwrap()));
            } else if content.starts_with("(") {
                return Some(Box::new(LCommand::new(content).unwrap()));
            } else {
                return Some(Box::new(CCommand::new(content).unwrap()));
            };
        }

        fn strip_comments(mut input: String) -> String {
            let comment_offset = input.find("//").unwrap_or(input.len());

            input.replace_range(comment_offset.., "");

            input
        }
        
    }

    fn get_content(raw_content: &str) -> Result<ALContent, ParseIntError> {
        let chars_to_trim: &[char] = &['(', ')', '@'];
        
        let content = raw_content.trim_matches(chars_to_trim);

        match content.parse::<u8>() {
            Ok(n) => return Ok(ALContent::DecimalValue(n)),
            Err(_) => {
                let content = String::from(content);
                return Ok(ALContent::Symbol(content));
            },
        }
    }

    // COMMANDS
    //
    #[derive(Debug)]
    enum ALContent {
        DecimalValue(u8),
        Symbol(String),
    }

    pub trait Command: std::fmt::Debug {}

    pub trait SymbolicCommand: Command {}

    impl<T: SymbolicCommand> Command for T {}

    #[derive(Debug)]
    pub struct ACommand {
        //raw_content: String,
        content: ALContent,
    }

    impl SymbolicCommand for ACommand {}

    impl ACommand {
        fn new(content: &str) -> Result<ACommand, ParseIntError> {
            //let raw_content = String::from(content);
            let content = get_content(content)?;

            Ok(ACommand { content })
        }
        
    }

    #[derive(Debug)]
    pub struct LCommand {
        //raw_content: String,
        content: ALContent,
    }

    impl SymbolicCommand for LCommand {}

    impl LCommand {
        fn new(content: &str) -> Result<LCommand, ParseIntError> {
            //let raw_content = String::from(content);
            let content = get_content(content)?;

            Ok(LCommand { content })
        }
    }

    #[derive(Debug)]
    pub struct CCommand {
        //content: String,
        dest: Option<String>,
        comp: Option<String>,
        jump: Option<String>,
    }

    impl Command for CCommand {}

    impl CCommand {
        fn new(content: &str) -> Result<CCommand, &'static str> {
            let contains_equals = content.contains('=');
            let contains_colon = content.contains(';');

            if contains_equals && contains_colon {
                // dest=comp;jump
                return Ok(CCommand {dest: None, comp: None, jump: None });
            } else if contains_equals {
                // dest=comp
                let v = content.split('=');
                let dest = v.next();
                let comp = v.next();

                return Ok(CCommand { dest, comp, jump: None });

            } else if contains_colon {
                // comp;jump
                return Ok(CCommand {dest: None, comp: None, jump: None });
            } else {
                return Err("Syntax error");
            }

            //let content = String::from(content);

            //Ok(CCommand { content })
        }
    }
}
