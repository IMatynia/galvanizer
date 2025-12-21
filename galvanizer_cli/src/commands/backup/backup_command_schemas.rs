use std::path::PathBuf;

use galvanizer_config::root_definition::RootDefinition;
use galvanizer_store::{snapshots::shapshot_delta::SnapshotDelta, store_error::StoreError};

use crate::commands::backup::walker_thread::WalkerError;

#[derive(Debug)]
pub enum BackupEvent {
    StartBackup {
        path: PathBuf,
        root: RootDefinition,
    },
    BackupError {
        error: StoreError,
        path: PathBuf,
        root: RootDefinition,
    },
    FinishBackup {
        path: PathBuf,
        root: RootDefinition,
    },
    WalkerError(WalkerError),
    NoChanges,
}

pub enum BackupJob {
    AddCoppiedEntry(SnapshotDelta),
    ProcessFileFurther { path: PathBuf, root: RootDefinition },
}
