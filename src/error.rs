
use std::{io, result, error, fmt};
use walkdir;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    cause: Option<Box<error::Error>>,
    message: String
}

#[derive(Debug)]
pub enum ErrorKind {
    FileSystemError,
    IOError,
}

impl error::Error for Error {
    fn description(&self) -> &str {
        self.message.as_ref()
    }
    fn cause(&self) -> Option<&error::Error> {
        self.cause.as_ref().map(AsRef::as_ref)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ::std::error::Error;
        writeln!(f, "{}", self.description())
    }
}

pub type Result<T> = result::Result<T, Error>;

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error {
            kind: ErrorKind::IOError,
            cause: Some(Box::new(e)),
            message: "Error interacting with file".to_owned()
        }
    }
}

impl From<walkdir::Error> for Error {
    fn from(e: walkdir::Error) -> Self {
        Error {
            kind: ErrorKind::FileSystemError,
            cause: Some(Box::new(e)),
            message: "Error interacting with file system".to_owned()
        }
    }
}
