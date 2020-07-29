use crate::error::{Error, ErrorKind, Result};
use regex::Regex;

/// A struct to hold configuration options used when running the assembler.
///
#[derive(Debug, PartialEq)]
pub struct Config {
    pub infile: String,
    pub outfile: String,
}

impl Config {
    /// The constructor method takes command line arguments, provided to it as an
    /// iterator that yields Strings.
    ///
    /// # Examples
    ///
    /// '''
    /// use std::env;
    /// use assembler::config::Config;
    ///
    /// fn main() {
    ///     // env::args() returns the arguments this program was started with
    ///     // as an 'Args' iterator that yields Strings.
    ///     let config = Config::new(env::args()).unwrap();
    /// }
    /// '''
    pub fn new<T>(mut args: T) -> Result<Config>
    where
        T: Iterator<Item = String>,
    {
        args.next();  // Ignore path of executable.

        let re_asm_ext = Regex::new(r"\.asm$").unwrap();

        let infile = match args.next() {
            Some(arg) => {
                if re_asm_ext.is_match(&arg[..]) {
                    arg
                } else {
                    return Err(Error::new(ErrorKind::InvalidInFileExt));
                }
            },
            None => return Err(Error::new(ErrorKind::MissingArguments)),
        };

        let outfile = match args.next() {
            Some(arg) => {
                let re_hack_ext = Regex::new(r"\.hack$").unwrap();

                if re_hack_ext.is_match(&arg[..]) {
                    arg
                } else {
                    return Err(Error::new(ErrorKind::InvalidOutFileExt));
                }
            }
            None => return Err(Error::new(ErrorKind::MissingOutputFilename)),
        };

        Ok(Config { infile, outfile })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_valid_config() {
        let mut args = vec![
            String::from("ignore/the/path"),
            String::from("test_input_file.asm"),
            String::from("test_output_file.hack"),
        ];

        let args = args.drain(..);

        assert_eq!(
            Config::new(args).unwrap(),
            Config {
                infile: String::from("test_input_file.asm"),
                outfile: String::from("test_output_file.hack"),
            }
        );
    }

    #[test]
    #[should_panic(expected = "invalid input file extension, only \\\'.asm\\\' accepted")]
    fn check_invalid_infilename() {
        let mut args = vec![
            String::from("ignore/the/path"),
            String::from("test_input_file.txt"),
            String::from("test_output_file.hack"),
        ];

        let args = args.drain(..);

        Config::new(args).unwrap();
    }

    #[test]
    #[should_panic(expected = "invalid output file extension, only \\\'.hack\\\' accepted")]
    fn check_invalid_outfilename() {
        let mut args = vec![
            String::from("ignore/the/path"),
            String::from("test_input_file.asm"),
            String::from("test_output_file.txt"),
        ];

        let args = args.drain(..);

        Config::new(args).unwrap();
    }

    #[test]
    #[should_panic(expected = "input and output filenames were not provided")]
    fn check_missing_args() {
        let mut args = vec![
            String::from("ignore/the/path"),
        ];

        let args = args.drain(..);

        Config::new(args).unwrap();
    }

    #[test]
    #[should_panic(expected = "output filename not provided")]
    fn check_missing_outfilename() {
        let mut args = vec![
            String::from("ignore/the/path"),
            String::from("test_input_file.asm"),
        ];

        let args = args.drain(..);

        Config::new(args).unwrap();
    }
}
