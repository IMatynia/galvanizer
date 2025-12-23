use crossbeam::channel::Receiver;
use galvanizer_store::{root_walker::WalkerError, store_error::StoreError};
use log::info;
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

pub fn backup_monitor_task(event_rx: Receiver<BackupEvent>) {
    while let Ok(event) = event_rx.recv() {
        info!("Backup event: {event:#?}");
    }
}
