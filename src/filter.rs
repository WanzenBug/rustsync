use ::{Source, Drain, SyncState, DrainStatus, SyncFileInfo, Result, SyncFileKind};

#[derive(Debug)]
pub enum SyncCompareResult {
    Sync,
    NoSync,
    NotComparable,
}

#[derive(Debug)]
pub struct SyncAllComparer {}

impl<S, D> SyncCompare<S, D> for SyncAllComparer where S: Source, D: Drain {

}

pub trait SyncCompare<S, D> where S: Source, D: Drain {
    fn compare(&self, source: &S, drain: &D, state: SyncState) -> SyncState {
        match state {
            SyncState::Ok(pair) => {
                let res = {
                    let drain_status = &pair.drain;
                    let sync_item = &pair.source;
                    match (drain_status, &sync_item.kind) {
                        (&DrainStatus::Exists,  &SyncFileKind::File) => self.compare_files(source, drain, sync_item),
                        (&DrainStatus::Exists, &SyncFileKind::Directory) => self.compare_directories(source, drain, sync_item),
                        (&DrainStatus::IsWrongKind(ref kind), _) => self.compare_mismatched_kinds(source, drain, sync_item, kind),
                        (&DrainStatus::Missing, _) =>self.compare_non_existing(source, drain, sync_item)
                    }
                };

                match res {
                    Ok(SyncCompareResult::Sync) => SyncState::Ok(pair),
                    Ok(SyncCompareResult::NoSync) => SyncState::Filtered(pair),
                    // TODO: Maybe make this error?
                    Ok(SyncCompareResult::NotComparable) => SyncState::Ok(pair),
                    Err(e) => SyncState::Error(e)
                }
            },
            SyncState::Filtered(pair) => SyncState::Filtered(pair),
            SyncState::Error(err) => SyncState::Error(err),
        }
    }

    fn compare_non_existing(&self, _source: &S, _drain: &D, _info: &SyncFileInfo) -> Result<SyncCompareResult> {
        Ok(SyncCompareResult::Sync)
    }

    fn compare_files(&self, _source: &S, _drain: &D, _info: &SyncFileInfo) -> Result<SyncCompareResult> {
        Ok(SyncCompareResult::Sync)
    }

    fn compare_directories(&self, _source: &S, _drain: &D, _info: &SyncFileInfo) -> Result<SyncCompareResult> {
        Ok(SyncCompareResult::Sync)
    }

    fn compare_mismatched_kinds(&self, _source: &S, _drain: &D, _info: &SyncFileInfo, _drain_kind: &SyncFileKind) -> Result<SyncCompareResult> {
        Ok(SyncCompareResult::NotComparable)
    }
}
