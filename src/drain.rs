use ::{Result, SyncFileKind, SyncFileInfo};
use std::{io};

pub trait Drain {
    fn get_info(&self, item: &SyncFileInfo) -> Result<DrainStatus>;
    fn create_file(&self, item: &SyncFileInfo) -> Result<Box<io::Write>>;
    fn create_directory(&self, item: &SyncFileInfo) -> Result<()>;
}

#[derive(Debug)]
pub enum DrainStatus {
    Exists,
    IsWrongKind(SyncFileKind),
    Missing,
}
