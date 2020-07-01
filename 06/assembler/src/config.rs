use crate::error::{Error, ErrorKind, Result};
use regex::Regex;

//struct Cacher<T>
//    where T: FnOnce() -> Regex
//{
//    calc: T,
//    value: Option<Regex>,
//}
//
//impl<T> Cacher<T>
//    where T: FnOnce() -> Regex
//{
//    fn new(calc: T) -> Cacher<T> {
//        Cacher {
//            calc,
//            value: None,
//        }
//    }
//
//    fn value(&mut self) -> &Regex {
//        match self.value {
//            Some(ref v) => v,
//            None => {
//                let v = (self.calc)();
//                self.value = Some(v);
//                self.value.as_ref().unwrap()
//            },
//        }
//    }
//}

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
        args.next();

        let re_asm_ext = Regex::new(r"\.asm$").unwrap();

        let infile = match args.next() {
            Some(arg) => {
                if re_asm_ext.is_match(&arg[..]) {
                    arg
                } else {
                    return Err(Error::new(ErrorKind::InvalidInFileExt));
                }
            },
            None => return Err(Error::new(ErrorKind::MissingInputFilename)),
        };

        let outfile = match args.next() {
            Some(arg) => {
                let re_hack_ext = Regex::new(r"\.hack$").unwrap();

                if re_hack_ext.is_match(&arg[..]) {
                    arg
                } else {
                    return Err(Error::new(ErrorKind::InvalidInFileExt));
                }
            }
            None => {
                re_asm_ext.replace(&infile[..], ".hack")
                    .into_owned()
            },
        };

        Ok(Config { infile, outfile })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_valid_config() {
        let arg_0 = String::from("ignore/the/path");
        let arg_1 = String::from("test_input_file.asm");
        let arg_2 = String::from("test_output_file.hack");
        let mut args = vec![arg_0, arg_1];
        let args = args.drain(..);

        assert_eq!(
            Config::new(args).unwrap(),
            Config {
                infile: String::from("test_input_file.asm"),
                outfile: String::from("test_output_file.hack"),
            }
        );
    }

//    #[test]
//    fn check_invalid_infilename() {
//        let arg_0 = String::from("ignore/the/path");
//        let arg_1 = String::from("testfile.txt");
//        let arg_2 = String::from("outfile.hack");
//        let mut args = vec![arg_0, arg_1];
//        let args = args.drain(..);
//
//        assert_eq!(
//            Config::new(args),
//            Error::new(ErrorKind::InvalidInFileExt)
//        );
//    }
}
