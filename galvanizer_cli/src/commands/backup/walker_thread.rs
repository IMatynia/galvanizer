use std::{io, thread::available_parallelism};

use crate::commands::backup::{
    backup_command_schemas::{BackupEvent, BackupJob},
    backup_root_walker::root_walker,
};
use crossbeam::channel::Sender;
use galvanizer_config::{Config, config::ConfigError};
use galvanizer_store::{
    snapshots::{shapshot_delta::SnapshotDelta, snapshot::Snapshot},
    store::Store,
};

#[derive(Debug)]
pub enum WalkerError {
    WalkDirError(walkdir::Error),
    RootConfigurationError(ConfigError),
    RootDoesNotExist(String),
    PathIdentifierError(&'static str),
    ModificationDateReadError(io::Error),
}

type WalkerResult<T> = Result<T, WalkerError>;

pub fn walker_thread_task(
    conifg: Config,
    old_snapshot: &Snapshot,
    store: Store,
    event_tx: Sender<BackupEvent>,
    snapshot_tx: Sender<SnapshotDelta>,
) {
    let cores = available_parallelism().map(|x| x.get()).unwrap_or(1);

    let worker_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(cores)
        .build()
        .expect("Could not build worker pool");

    for job in root_walker(&conifg, old_snapshot) {
        match job {
            Ok(job) => {
                let store_clone = store.clone();
                let event_tx_clone = event_tx.clone();
                let snapshot_tx_clone = snapshot_tx.clone();

                // Worker closure
                worker_pool.spawn(move || match job {
                    BackupJob::AddCoppiedEntry(snapshot_delta) => {
                        event_tx_clone.send(BackupEvent::NoChanges);
                        snapshot_tx_clone.send(snapshot_delta);
                    }
                    BackupJob::ProcessFileFurther { path, root } => {
                        event_tx_clone.send(BackupEvent::StartBackup {
                            path: path.clone(),
                            root: root.clone(),
                        });
                        match store_clone.store_file(&path, &root) {
                            Ok(delta) => {
                                event_tx_clone.send(BackupEvent::FinishBackup { path, root });
                                snapshot_tx_clone.send(delta);
                            }
                            Err(e) => {
                                event_tx_clone.send(BackupEvent::BackupError {
                                    error: e,
                                    path,
                                    root,
                                });
                            }
                        };
                    }
                });
            }
            Err(e) => {
                event_tx.send(BackupEvent::WalkerError(e));
            }
        }
    }
}
