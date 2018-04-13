use ::{Result, Drain, SyncFileInfo, SyncFileKind, DrainStatus};
use std::{path, fs, io};

#[derive(Debug)]
pub struct LocalDrain {
    root: path::PathBuf
}

impl LocalDrain {
    pub fn new<A: AsRef<path::Path>>(root: A) -> Self {
        LocalDrain {
            root: root.as_ref().to_owned()
        }
    }

    fn join<A: AsRef<path::Path>>(&self, path: A) -> path::PathBuf {
        self.root.join(path)
    }
}

impl Drain for LocalDrain {
    fn get_info(&self, item: &SyncFileInfo) -> Result<DrainStatus> {
        let full_path = self.join(&item.rel_path);
        if full_path.exists() {
            match (full_path.is_file(), full_path.is_dir(), &item.kind) {
                (true, false, &SyncFileKind::File) => Ok(DrainStatus::Exists),
                (false, true, &SyncFileKind::Directory) => Ok(DrainStatus::Exists),
                (file, dir, _) => {
                    let info = if file {
                        SyncFileKind::File
                    } else if dir {
                        SyncFileKind::Directory
                    } else {
                        unimplemented!()
                    };
                    Ok(DrainStatus::IsWrongKind(info))
                }
            }
        } else {
            Ok(DrainStatus::Missing)
        }
    }

    fn create_file(&self, item: &SyncFileInfo) -> Result<Box<io::Write>> {
        let full_path = self.join(&item.rel_path);
        let file = fs::File::create(full_path)?;
        Ok(Box::new(file))
    }

    fn create_directory(&self, item: &SyncFileInfo) -> Result<()> {
        let full_path = self.join(&item.rel_path);
        if full_path.exists() {
            Ok(())
        } else {
            fs::create_dir(full_path).map_err(Into::into)
        }
    }
}
