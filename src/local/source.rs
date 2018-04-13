use ::{Result, Source, walkdir, SyncFileInfo, SyncFileKind};
use std::{path, fs, io};

#[derive(Debug)]
pub struct LocalSource {
    root: path::PathBuf,
}

impl LocalSource {
    pub fn new<A: AsRef<path::Path>>(root: A) -> Self {
        LocalSource {
            root: root.as_ref().to_owned()
        }
    }
}

impl Source for LocalSource {
    fn candidates(&self) -> Box<Iterator<Item=Result<SyncFileInfo>>> {
        let path_root = self.root.clone();
        let iter = walkdir::WalkDir::new(&self.root).into_iter()
            .map(move |result| {
                let root = &path_root;
                result.map(|walkdir_entry|
                    walkdir_entry_to_info(walkdir_entry, root.as_ref())
                ).map_err(Into::into)
            });

        Box::new(iter)
    }

    fn get_reader(&self, item: &SyncFileInfo) -> Result<Box<io::Read>> {
        let path = self.root.join(&item.rel_path);
        let file = fs::File::open(path)?;
        Ok(Box::new(io::BufReader::new(file)))
    }
}

fn walkdir_entry_to_info(entry: walkdir::DirEntry, root: &path::Path) -> SyncFileInfo {
    let rel_path = entry.path().strip_prefix(root).unwrap_or_else(|_| entry.path()).to_owned();
    let file_type = entry.file_type();
    let kind = if file_type.is_file() {
        SyncFileKind::File
    } else if file_type.is_dir() {
        SyncFileKind::Directory
    } else {
        unimplemented!()
    };

    SyncFileInfo {
        rel_path,
        kind,
    }
}
