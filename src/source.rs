use ::{Result, SyncFileInfo};
use std::{io};

pub trait Source {
    fn candidates(&self) -> Box<Iterator<Item=Result<SyncFileInfo>>>;
    fn get_reader(&self, item: &SyncFileInfo) -> Result<Box<io::Read>>;
}
