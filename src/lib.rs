extern crate walkdir;

mod sync_info;
pub mod local;
pub mod error;
pub mod source;
pub mod drain;
pub mod filter;

pub use drain::{Drain, DrainStatus};
pub use source::Source;
pub use error::{Error, Result};
pub use sync_info::{SyncFileInfo, SyncFileKind};

#[derive(Debug)]
pub struct SyncInfoPair {
    source: SyncFileInfo,
    drain: DrainStatus,
}

#[derive(Debug)]
pub enum SyncState {
    Ok(SyncInfoPair),
    Filtered(SyncInfoPair),
    Error(Error),
}

#[derive(Debug)]
pub struct Syncer<S, D> where S: Source, D: Drain {
    source: S,
    drain: D,
}

impl<S, D> Syncer<S, D> where S: Source, D: Drain {
    pub fn new(source: S, drain: D) -> Self {
        Syncer {
            source,
            drain
        }
    }

    pub fn sync(&self) -> Result<()> {
        use filter::SyncCompare;

        let source_entries = self.source.candidates();
        let cmp = filter::SyncAllComparer {};

        let mapped = source_entries.into_iter()
            .map(|r| {
                match r {
                    Ok(source) => {
                        match self.drain.get_info(&source) {
                            Ok(drain) => SyncState::Ok(SyncInfoPair { source, drain }),
                            Err(e) => SyncState::Error(e),
                        }
                    },
                    Err(e) => {
                        SyncState::Error(e)
                    }
                }
            })
            .map(move |state| {
                cmp.compare(&self.source, &self.drain, state)
            });

        for entry in mapped {
            match entry {
                SyncState::Ok(pair) => {
                    eprintln!("Syncing {:#?}, {:#?}", pair.source, pair.drain);
                    copy(&self.source, &self.drain, &pair.source)?;
                },
                SyncState::Filtered(pair) => {
                    eprintln!("Skipping {:#?}", pair.source);
                }
                SyncState::Error(e) => {
                    eprintln!("Error while syncing: {}", e);
                }
            }
        }
        Ok(())
    }
}

fn copy<S, D>(source: &S, drain: &D, info: &SyncFileInfo) -> Result<()> where S: Source, D: Drain {
    match info.kind {
        SyncFileKind::Directory => {
            drain.create_directory(info)
        },
        SyncFileKind::File => {
            let mut writer = drain.create_file(info)?;
            let mut reader = source.get_reader(info)?;
            let _ = std::io::copy(&mut reader, &mut writer)?;
            Ok(())
        }
    }
}
