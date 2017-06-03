use ::{Result, SyncableInfo, DrainPathStatus, Drain, DrainSyncable, DrainNotExist};
use std::{path, fs, sync, io};

pub struct LocalDrainSyncable {
    full_path: path::PathBuf,
    prefix: sync::Arc<path::PathBuf>,
}

impl DrainSyncable for LocalDrainSyncable {
    type Writer = io::BufWriter<fs::File>;

    fn info(&self) -> Result<SyncableInfo> {
        let meta = fs::metadata(&self.full_path)?;
        self.full_path
            .strip_prefix(self.prefix.as_ref())
            .map(|pref| {
                if self.full_path.is_dir() {
                    SyncableInfo::Directory(pref.components())
                } else {
                    SyncableInfo::File{ components: pref.components(), size: meta.len()}
                }
            })
            .map_err(Into::into)
    }

    fn into_file(self) -> Result<Self::Writer> {
        if self.full_path.is_dir() {
            fs::remove_dir_all(&self.full_path)?;
        }
        let f = fs::File::create(&self.full_path)?;
        Ok(io::BufWriter::new(f))
    }

    fn into_dir(self) -> Result<()> {
        if self.full_path.is_dir() {
            Ok(())
        } else {
            fs::remove_file(&self.full_path)?;
            fs::DirBuilder::new()
                .recursive(true)
                .create(&self.full_path)
                .map_err(Into::into)
        }
    }
}

pub struct LocalNoFile {
    path: path::PathBuf
}

impl DrainNotExist for LocalNoFile {
    type Writer = io::BufWriter<fs::File>;

    fn dir(self) -> Result<()> {
        fs::DirBuilder::new()
            .recursive(true)
            .create(&self.path)
            .map_err(Into::into)
    }

    fn file(self) -> Result<Self::Writer> {
        if let Some(dir) = self.path.parent() {
            fs::DirBuilder::new()
                .recursive(true)
                .create(dir)?;
        }
        let f = fs::File::create(&self.path)?;
        Ok(io::BufWriter::new(f))
    }
}

pub struct LocalDrain {
    base: sync::Arc<path::PathBuf>,
}

impl LocalDrain {
    pub fn new<P: AsRef<path::Path>>(path: P) -> Self {
        LocalDrain {
            base: sync::Arc::new(path.as_ref().to_owned())
        }
    }
}

impl Drain for LocalDrain {
    type Syncable = LocalDrainSyncable;

    type NoFile = LocalNoFile;

    fn get_file_info(&self, path: &path::Path) -> Result<DrainPathStatus<Self::Syncable, Self::NoFile>> {
        let full_path = self.base.as_ref().join(path);
        if full_path.exists() {
            Ok(DrainPathStatus::Exists(LocalDrainSyncable {
                full_path: full_path,
                prefix: self.base.clone(),
            }))
        } else {
            Ok(DrainPathStatus::NoPath(LocalNoFile { path: full_path }))
        }
    }
}