
use std::{io, result, path};
use walkdir;

#[derive(Debug, Error)]
pub enum Error {
    FilePath(walkdir::Error),
    IOError(io::Error),
    PathConversionError(path::StripPrefixError),
    #[error(msg_embedded, non_std, no_from)]
    EncodingError(String),
}

pub type Result<T> = result::Result<T, Error>;