use ::{Result, Error};
use std::path::Path;

pub fn path_to_str(p: &Path) -> Result<&str> {
    p.to_str().ok_or_else(|| Error::EncodingError(format!("Path {:?} could not be encoded using UTF-8", p)))
}