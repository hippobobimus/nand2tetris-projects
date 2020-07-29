use std::io;
use std::num::ParseIntError;
use std::fmt;
use std::error;
use std::result;

/// A specialised 'Result' type for assembler operations.
///
/// This type is used throughout 'assembler' for any operations which
/// produce an error.
///
/// The typedef is used to avoid writing out the 'Error' explicitly and is
/// otherwise a direct mapping to 'Result'.
///
pub type Result<T> = result::Result<T, Error>;

/// The error type for assembler operations.
///
#[derive(Debug)]
pub struct Error {
    repr: Repr,
}

impl Error {
    /// Creates a new assembler error from a known kind of error.
    ///
    /// Used to create errors which do not originate as a std::num::ParseIntError or from the
    /// std::io library.
    ///
    pub fn new(error_kind: ErrorKind) -> Error {
        Error { repr: Repr::Other(error_kind.as_str()) }
    }
}

#[derive(Debug)]
enum Repr {
    IO(io::Error),
    ParseInt(ParseIntError),
    Other(&'static str),
}

/// General categories of assembler error.
///
pub enum ErrorKind {
    /// The parser has advanced through all lines of the input BufReader.
    EndOfFile,
    /// The function call failed because it cannot take a Command of this type.
    InvalidCmdType,
    /// The provided input path contains a file extension that is not accepted.
    InvalidInFileExt,
    /// The provided output path contains a file extension that is not accepted.
    InvalidOutFileExt,
    /// A syntax error in the Hack assembly instruction has been identified.
    InvalidSyntax,
    /// An insufficient number of arguments were provided when generating a Config instance,
    MissingArguments,
    /// An output filename was not provided when generating a Config instance.
    MissingOutputFilename,
    /// An attempt to add a new variable to the SymbolTable has failed because there are no more
    /// available RAM addresses.
    RAMFull,
    /// An attempt to add a new variable or label to the SymbolTable has failed because it is
    /// already present.
    SymbolExists,
}

impl ErrorKind {
    fn as_str(&self) -> &'static str {
        match *self {
            ErrorKind::EndOfFile => "the end of the file has been reached",
            ErrorKind::InvalidSyntax => "invalid syntax",
            ErrorKind::InvalidCmdType => "this function cannot act on Commands of this type",
            ErrorKind::MissingArguments => "input and output filenames were not provided",
            ErrorKind::MissingOutputFilename => "output filename not provided",
            ErrorKind::InvalidInFileExt => "invalid input file extension, only '.asm' accepted",
            ErrorKind::InvalidOutFileExt => "invalid output file extension, only '.hack' accepted",
            ErrorKind::SymbolExists => "this symbol has already been defined",
            ErrorKind::RAMFull => "there are no more free RAM addresses",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.repr {
            Repr::IO(ref e) => e.fmt(f),
            Repr::ParseInt(ref e) => e.fmt(f),
            Repr::Other(ref e) =>
                write!(f, "Error: {}", e),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self.repr {
            Repr::IO(ref e) => Some(e),
            Repr::ParseInt(ref e) => Some(e),
            Repr::Other(_) => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error { repr: Repr::IO(err) }
    }
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Error {
        Error { repr: Repr::ParseInt(err) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_debug_error() {
        let error = Error::new(ErrorKind::RAMFull);

        let expected = "Error { repr: Other(\"there are no more free RAM addresses\") }";

        assert_eq!(format!("{:?}", error), expected);
    }
}
