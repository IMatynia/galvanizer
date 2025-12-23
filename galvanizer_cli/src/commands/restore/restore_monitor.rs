use crossbeam::channel::Receiver;
use galvanizer_store::{
    snapshot_walker::SnapshotWalkerErr, store_cache_data_handler::CacheHandlerError,
};
use log::info;
use std::path::PathBuf;

#[derive(Debug)]
pub enum RestoreEvent {
    StartRestore {
        worker: String,
        destination: PathBuf,
    },
    RestoreError {
        worker: String,
        error: CacheHandlerError,
        destination: PathBuf,
    },
    FinishRestore {
        worker: String,
        bytes_restored: u64,
        destination: PathBuf,
    },
    WalkerError(SnapshotWalkerErr),
    NoChanges {
        worker: String,
        destination: PathBuf,
    },
}

pub fn restore_monitor_task(event_rx: Receiver<RestoreEvent>) {
    while let Ok(event) = event_rx.recv() {
        info!("Backup event: {event:#?}");
    }
}
