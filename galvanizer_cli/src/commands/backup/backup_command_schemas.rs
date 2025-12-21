use galvanizer_store::{root_walker::WalkerError, store_error::StoreError};
use std::path::PathBuf;

#[derive(Debug)]
pub enum BackupEvent {
    StartBackup {
        worker: String,
        path: PathBuf,
        root_id: String,
    },
    BackupError {
        worker: String,
        error: StoreError,
        path: PathBuf,
        root_id: String,
    },
    FinishBackup {
        worker: String,
        path: PathBuf,
        root_id: String,
    },
    WalkerError(WalkerError),
    NoChanges {
        worker: String,
        path_id: String,
        root_id: String,
    },
}
