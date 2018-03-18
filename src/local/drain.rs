use {futures, BasicFileInfo, Error, FileType, SyncDrain, SyncFuture};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct LocalDrain {
    root: PathBuf,
}

impl LocalDrain {
    pub fn new<A: AsRef<Path>>(source_location: A) -> LocalDrain {
        let root = source_location.as_ref().to_owned();
        LocalDrain { root }
    }
}

impl SyncDrain for LocalDrain {
    fn check_destination(&self, info: BasicFileInfo) -> SyncFuture<Option<BasicFileInfo>> {
        let full_path = self.root.join(&info.path);

        if full_path.exists() {
            let kind = if full_path.is_file() {
                FileType::File
            } else {
                FileType::Directory
            };

            Box::new(futures::future::ok(Some(BasicFileInfo {
                path: info.path,

                kind,
            })))
        } else {
            Box::new(futures::future::ok(None))
        }
    }

    fn write_file(&self, info: BasicFileInfo) -> SyncFuture<Box<futures::io::AsyncWrite>> {
        Box::new(futures::future::err(Error::new("Not implemented")))
    }
}
