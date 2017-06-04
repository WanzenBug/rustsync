extern crate walkdir;

#[macro_use]
extern crate derive_error;

mod util;
pub mod local;
pub mod error;
pub mod source;
pub mod drain;

pub use drain::{DrainSyncable, Drain, DrainNotExist, DrainPathStatus};
pub use source::{Source, SourceSyncable, MultiSource};
pub use error::{Error, Result};
use std::path::{Path, Components};
use std::io;

pub struct SyncConfig {}

impl SyncConfig {}

impl Default for SyncConfig {
    fn default() -> Self {
        SyncConfig {}
    }
}

pub fn sync<S, D, P>(source: S, drain: D, update: P) -> Result<()>
    where S: Source, D: Drain, P: SyncProgress {
    sync_with_config(source, drain, &SyncConfig::default(), update)
}

pub fn sync_with_config<S, D, P>(mut source: S, drain: D, config: &SyncConfig, mut updates: P) -> Result<()> where S: Source, D: Drain, P: SyncProgress {
    let iter = source.iter_file_info()?;
    for f in iter {
        let mut file_handle: <S as Source>::Syncable = f?;
        let (dest, is_file, updates) = {
            let info = file_handle.info()?;
            let file_updates = updates.next_sync(&info);

            let dest = drain.get_file_info(info.get_path())?;
            if let SyncableInfo::File { .. } = info {
                (dest, true, file_updates)
            } else {
                (dest, false, file_updates)
            }
        };

        match (dest, is_file) {
            (DrainPathStatus::Exists(dest_info), true) => {
                copy(file_handle.reader()?, dest_info.into_file()?, updates)?;
            }
            (DrainPathStatus::Exists(dest_info), false) => {
                dest_info.into_dir()?;
            }
            (DrainPathStatus::NoPath(no_path), true) => {
                copy(file_handle.reader()?, no_path.file()?, updates)?;
            }
            (DrainPathStatus::NoPath(no_path), false) => {
                no_path.dir()?;
            }
        };
    }

    Ok(())
}

const DEFAULT_BUFFER_SIZE: usize = 4096;

fn copy<R: io::Read, W: io::Write, F: FileProgress>(mut reader: R, mut writer: W, mut progress: F) -> io::Result<u64> {
    let mut buf: [u8; DEFAULT_BUFFER_SIZE] = [0; DEFAULT_BUFFER_SIZE];
    let mut written = 0;
    loop {
        let len = match reader.read(&mut buf) {
            Ok(0) => return Ok(written),
            Ok(len) => len,
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        writer.write_all(&buf[..len])?;
        written += len as u64;
        progress.update(written)
    }
}

pub struct  SuppressUpdate {}

impl SyncProgress for SuppressUpdate {
    type FileProgress = SuppressUpdate;

    fn next_sync<'a>(&mut self, _: &SyncableInfo<'a>) -> Self::FileProgress {
        SuppressUpdate {}
    }
}

impl FileProgress for SuppressUpdate {
    fn update(&mut self, _: u64) {
    }
}

pub trait SyncProgress {
    type FileProgress: Send + FileProgress;

    fn next_sync<'a>(&mut self, item: &SyncableInfo<'a>) -> Self::FileProgress;
}

pub trait FileProgress {
    fn update(&mut self, transferred: u64);
}

#[derive(Debug)]
pub enum SyncableInfo<'a> {
    File { components: Components<'a>, size: u64 },
    Directory(Components<'a>),
}

impl<'a> SyncableInfo<'a> {
    pub fn get_path(&self) -> &Path {
        match *self {
            SyncableInfo::File { components: ref x, .. } | SyncableInfo::Directory(ref x) => x.as_ref()
        }
    }
}