use std::thread::available_parallelism;

use crossbeam::channel::Sender;
use galvanizer_config::Config;
use galvanizer_store::{
    snapshot_walker::{self, SnapshotWalkerFilter, SnapshotWalkerItem},
    snapshots::snapshot::Snapshot,
    store_cache::file_io::restore_file,
};

use crate::commands::restore::restore_monitor::RestoreEvent;

pub fn snapshot_walker_task(
    config: Config,
    event_tx: Sender<RestoreEvent>,
    snapshot: Snapshot,
    restoration_options: SnapshotWalkerFilter,
) {
    let cores = available_parallelism().map(|x| x.get()).unwrap_or(1);

    let worker_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(cores)
        .thread_name(|id| format!("Worker-{id}"))
        .build()
        .expect("Could not build worker pool");

    for job in snapshot_walker::walk_all_files_in_snapshot_filtered(&snapshot, &restoration_options)
    {
        match job {
            Ok(job_detail) => {
                let event_tx = event_tx.clone();
                let config = config.clone();

                let SnapshotWalkerItem {
                    root_id,
                    path: destination,
                    hash_str,
                } = job_detail;

                worker_pool.spawn(move || {
                    let worker_id = std::thread::current().name().unwrap_or("MAIN").to_string();
                    let _ = event_tx.send(RestoreEvent::StartRestore {
                        root_id: root_id.clone(),
                        worker: worker_id.clone(),
                        destination: destination.clone(),
                    });

                    match restore_file(&config, &destination, &hash_str) {
                        Ok(bytes_restored) => {
                            let _ = event_tx.send(RestoreEvent::FinishRestore {
                                root_id,
                                worker: worker_id,
                                bytes_restored,
                                destination,
                            });
                        }
                        Err(e) => {
                            let _ = event_tx.send(RestoreEvent::RestoreError {
                                root_id,
                                worker: worker_id,
                                error: e,
                                destination,
                            });
                        }
                    }
                });
            }
            Err(e) => {
                let _ = event_tx.send(RestoreEvent::WalkerError(e));
            }
        }
    }
}
