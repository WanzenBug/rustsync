use std::fmt::{Display, Formatter, Result as FmtResult};

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]

pub struct Error {
    kind: ErrorKind,

    cause: Option<Box<::std::error::Error + Send>>,

    message: String,
}

impl Error {
    pub fn new<S: Into<String>>(s: S) -> Error {
        Error {
            kind: ErrorKind::Generic,

            cause: None,

            message: s.into(),
        }
    }
}

#[derive(Debug)]

pub enum ErrorKind {
    Generic,

    DirWalk,

    SyncronizationError,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        use std::error::Error;

        writeln!(f, "{}", self.description())
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        &self.message
    }

    fn cause(&self) -> Option<&::std::error::Error> {
        use std::ops::Deref;

        if let Some(ref e) = self.cause {
            Some(e.deref())
        } else {
            None
        }
    }
}

impl From<::walkdir::Error> for Error {
    fn from(e: ::walkdir::Error) -> Self {
        Error {
            message: "Error crawling directories".to_owned(),

            kind: ErrorKind::DirWalk,

            cause: Some(Box::new(e)),
        }
    }
}

impl<T> From<::std::sync::mpsc::SendError<T>> for Error
where
    T: 'static + Send,
{
    fn from(e: ::std::sync::mpsc::SendError<T>) -> Self {
        Error {
            message: "Error syncing between threads".to_owned(),

            kind: ErrorKind::SyncronizationError,

            cause: Some(Box::new(e)),
        }
    }
}
