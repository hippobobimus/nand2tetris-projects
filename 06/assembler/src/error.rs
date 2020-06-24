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

pub enum ErrorKind {
    InvalidSyntax,
    MissingInputFilename,
    InvalidInFileExt,
    InvalidOutFileExt,
}

impl ErrorKind {
    fn as_str(&self) -> &'static str {
        match *self {
            ErrorKind::InvalidSyntax => "invalid syntax",
            ErrorKind::MissingInputFilename => "input filename not provided",
            ErrorKind::InvalidInFileExt => "invalid input file extension, only '.asm' accepted",
            ErrorKind::InvalidOutFileExt => "invalid output file extension, only '.hack' accepted",
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
