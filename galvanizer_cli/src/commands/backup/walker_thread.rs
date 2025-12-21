use std::thread::available_parallelism;

use crate::commands::backup::backup_command_schemas::BackupEvent;
use crossbeam::channel::Sender;
use galvanizer_config::Config;
use galvanizer_store::{
    root_walker::{WalkResult, walk_all_files_in_backup_roots},
    snapshots::{shapshot_delta::SnapshotDelta, snapshot::Snapshot},
    store::Store,
};

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

    for job in walk_all_files_in_backup_roots(&conifg, old_snapshot) {
        match job {
            Ok(job) => {
                let store_clone = store.clone();
                let event_tx_clone = event_tx.clone();
                let snapshot_tx_clone = snapshot_tx.clone();

                // Worker closure
                worker_pool.spawn(move || match job {
                    WalkResult::UnchangedFile(snapshot_delta) => {
                        let _ = event_tx_clone.send(BackupEvent::NoChanges);
                        let _ = snapshot_tx_clone.send(snapshot_delta);
                    }
                    WalkResult::NewFile { path, root } => {
                        let _ = event_tx_clone.send(BackupEvent::StartBackup {
                            path: path.clone(),
                            root_id: root.name().to_string(),
                        });
                        match store_clone.store_file(&path, &root) {
                            Ok(delta) => {
                                let _ = event_tx_clone.send(BackupEvent::FinishBackup {
                                    path,
                                    root_id: root.name().to_string(),
                                });
                                let _ = snapshot_tx_clone.send(delta);
                            }
                            Err(e) => {
                                let _ = event_tx_clone.send(BackupEvent::BackupError {
                                    error: e,
                                    path,
                                    root_id: root.name().to_string(),
                                });
                            }
                        };
                    }
                });
            }
            Err(e) => {
                let _ = event_tx.send(BackupEvent::WalkerError(e));
            }
        }
    }
}
