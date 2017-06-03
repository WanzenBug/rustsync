use ::{Result, Source, SourceSyncable, SyncableInfo, walkdir, util};
use std::{path, fs, io};

pub struct LocalSourceSyncable {
    relative_path: String,
    full_path: path::PathBuf,
}

impl SourceSyncable for LocalSourceSyncable {
    type Reader = io::BufReader<fs::File>;

    fn info(&self) -> Result<SyncableInfo> {
        let meta = fs::metadata(&self.full_path)?;
        let size = meta.len();
        let full_path: &path::Path = self.full_path.as_ref();
        let rel_path: &path::Path = self.relative_path.as_ref();
        if full_path.is_dir() {
            Ok(SyncableInfo::Directory(rel_path.components()))
        } else {
            Ok(SyncableInfo::File { components: rel_path.components(), size: size })
        }
    }

    fn reader(&mut self) -> Result<Self::Reader> {
        let f = fs::File::open(&self.full_path)?;
        Ok(io::BufReader::new(f))
    }
}

pub struct LocalSource {
    source_path: path::PathBuf,
    strip_root_dir: bool,
}

impl LocalSource {
    pub fn new<P: AsRef<path::Path>>(path: P) -> Self {
        Self::with_strip_root_dir(path, false)
    }

    pub fn with_strip_root_dir<P: AsRef<path::Path>>(path: P, strip_root: bool) -> Self {
        LocalSource {
            source_path: path.as_ref().to_owned(),
            strip_root_dir: strip_root,
        }
    }
}

pub struct LocalSourceIterator {
    prefix: path::PathBuf,
    iter: walkdir::Iter,
}

impl LocalSourceIterator {
    fn new(source: &path::Path, strip_root: bool) -> Self {
        let prefix = if strip_root {
            source
        } else {
            source.parent().unwrap_or(source)
        };

        LocalSourceIterator {
            prefix: prefix.to_owned(),
            iter: walkdir::WalkDir::new(source).into_iter()
        }
    }
}

impl Iterator for LocalSourceIterator {
    type Item = Result<LocalSourceSyncable>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(Ok(x)) => {
                let path: path::PathBuf = x.path().to_owned();
                let relative = match path.strip_prefix(&self.prefix)
                    .map_err(Into::into)
                    .and_then(util::path_to_str) {
                    Ok(p) => p.to_owned(),
                    Err(e) => return Some(Err(e.into()))
                };

                Some(Ok(LocalSourceSyncable {
                    full_path: path,
                    relative_path: relative
                }))
            }
            Some(Err(e)) => Some(Err(e.into())),
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl Source for LocalSource {
    fn iter_file_info(&mut self) -> Result<Self::Iter> {
        Ok(LocalSourceIterator::new(self.source_path.as_ref(), self.strip_root_dir))
    }
    type Iter = LocalSourceIterator;
    type Syncable = LocalSourceSyncable;
}
