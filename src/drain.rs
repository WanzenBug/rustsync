use std::{io, path};
use ::{Result, SyncableInfo};

pub trait DrainSyncable {
    type Writer: io::Write;

    fn info(&self) -> Result<SyncableInfo>;
    fn into_file(self) -> Result<Self::Writer>;
    fn into_dir(self) -> Result<()>;
}

pub enum DrainPathStatus<S, N> where S: DrainSyncable, N: DrainNotExist {
    Exists(S),
    NoPath(N),
}

pub trait DrainNotExist {
    type Writer: io::Write;

    fn dir(self) -> Result<()>;
    fn file(self) -> Result<Self::Writer>;
}

pub trait Drain {
    type Syncable: DrainSyncable;
    type NoFile: DrainNotExist;

    fn get_file_info(&self, path: &path::Path) -> Result<DrainPathStatus<Self::Syncable, Self::NoFile>>;
}