
use ::std::path::PathBuf;

#[derive(Debug, Eq, PartialEq)]
pub enum SyncFileKind {
    File,
    Directory,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SyncFileInfo {
    pub rel_path: PathBuf,
    pub kind: SyncFileKind
}
