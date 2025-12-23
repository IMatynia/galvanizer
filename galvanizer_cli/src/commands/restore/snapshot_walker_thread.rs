use std::thread::available_parallelism;

use crossbeam::channel::Sender;
use galvanizer_config::Config;
use galvanizer_store::{
    snapshot_walker::{self, RestoreArgs},
    snapshots::snapshot::Snapshot,
    store_cache_data_handler::uncompress_and_restore,
};

use crate::commands::restore::restore_monitor::RestoreEvent;

pub fn snapshot_walker_task(
    config: Config,
    event_tx: Sender<RestoreEvent>,
    snapshot: Snapshot,
    restoration_options: RestoreArgs,
) {
    let cores = available_parallelism().map(|x| x.get()).unwrap_or(1);

    let worker_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(cores)
        .thread_name(|id| format!("Worker-{id}"))
        .build()
        .expect("Could not build worker pool");

    for job in snapshot_walker::walk_all_files_in_snapshot(&config, &snapshot, &restoration_options)
    {
        match job {
            Ok(job_details) => {
                let event_tx = event_tx.clone();
                let config = config.clone();

                worker_pool.spawn(move || {
                    let worker_id = std::thread::current().name().unwrap_or("MAIN").to_string();
                    match job_details {
                        snapshot_walker::SnapshotWalkerJob::RestoreFile {
                            destination,
                            hash_str,
                        } => {
                            let _ = event_tx.send(RestoreEvent::StartRestore {
                                worker: worker_id.clone(),
                                destination: destination.clone(),
                            });

                            match uncompress_and_restore(&config, &destination, &hash_str) {
                                Ok(bytes_restored) => {
                                    let _ = event_tx.send(RestoreEvent::FinishRestore {
                                        worker: worker_id,
                                        bytes_restored,
                                        destination,
                                    });
                                }
                                Err(e) => {
                                    let _ = event_tx.send(RestoreEvent::RestoreError {
                                        worker: worker_id,
                                        error: e,
                                        destination,
                                    });
                                }
                            }
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
