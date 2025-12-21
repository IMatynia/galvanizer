use galvanizer_store::{root_walker::WalkerError, store_error::StoreError};
use std::path::PathBuf;

#[derive(Debug)]
pub enum BackupEvent {
    StartBackup {
        path: PathBuf,
        root_id: String,
    },
    BackupError {
        error: StoreError,
        path: PathBuf,
        root_id: String,
    },
    FinishBackup {
        path: PathBuf,
        root_id: String,
    },
    WalkerError(WalkerError),
    NoChanges,
}
