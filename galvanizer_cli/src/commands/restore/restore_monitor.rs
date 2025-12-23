use crossbeam::channel::Receiver;
use galvanizer_store::{
    snapshot_walker::SnapshotWalkerErr, store_cache::file_io::CacheHandlerError,
};
use log::info;
use std::path::PathBuf;

#[derive(Debug)]
pub enum RestoreEvent {
    StartRestore {
        worker: String,
        root_id: String,
        destination: PathBuf,
    },
    RestoreError {
        worker: String,
        error: CacheHandlerError,
        root_id: String,
        destination: PathBuf,
    },
    FinishRestore {
        worker: String,
        bytes_restored: u64,
        root_id: String,
        destination: PathBuf,
    },
    WalkerError(SnapshotWalkerErr),
}

pub fn restore_monitor_task(event_rx: Receiver<RestoreEvent>) {
    while let Ok(event) = event_rx.recv() {
        info!("Backup event: {event:#?}");
    }
}
