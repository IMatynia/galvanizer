use galvanizer_config::{Config, config::ConfigError, root_definition::RootDefinition};
use galvanizer_store::{
    snapshots::{shapshot_delta::SnapshotDelta, snapshot::Snapshot},
    store::StoreBuilder,
    store_cache_tools::{get_file_last_modified_date, get_path_identifier},
    store_error::StoreError,
};
use rayon::iter::Either;

use crate::cli_errors::{CLIError, CLIResult};
use crossbeam::thread;
use std::{io, path::PathBuf, sync::Arc, thread::available_parallelism};
use walkdir::WalkDir;

#[derive(Debug)]
pub enum WalkerError {
    WalkDirError(walkdir::Error),
    RootConfigurationError(ConfigError),
    RootDoesNotExist(String),
    PathIdentifierError(&'static str),
    ModificationDateReadError(io::Error),
}

type WalkerResult<T> = Result<T, WalkerError>;

pub enum BackupJob {
    AddCoppiedEntry(SnapshotDelta),
    ProcessFileFurther { path: PathBuf, root: RootDefinition },
}

pub fn root_walker<'a>(
    config: &'a Config,
    old_snapshot: &'a Snapshot,
) -> impl Iterator<Item = Result<BackupJob, WalkerError>> + 'a {
    config.backup_roots().iter().flat_map(move |root| {
        let root = root.clone();

        let root_prefs = match config.get_combined_preferences_for_root(root.name()) {
            Some(p) => p,
            None => {
                return Either::Left(std::iter::once(Err(WalkerError::RootDoesNotExist(
                    root.name().to_string(),
                ))));
            }
        };

        let include_set = match root_prefs.inclusions_globset() {
            Ok(s) => s,
            Err(e) => {
                return Either::Left(std::iter::once(Err(WalkerError::RootConfigurationError(e))));
            }
        };

        let exclude_set = match root_prefs.exclusions_globset() {
            Ok(s) => s,
            Err(e) => {
                return Either::Left(std::iter::once(Err(WalkerError::RootConfigurationError(e))));
            }
        };

        Either::Right(
            WalkDir::new(root.path())
                .into_iter()
                .filter_map(move |dir_entry| {
                    let dir_entry = match dir_entry {
                        Ok(e) => e,
                        Err(e) => return Some(Err(WalkerError::WalkDirError(e))),
                    };
                    // Silent skip - not a file
                    if !dir_entry.file_type().is_file() {
                        return None;
                    }
                    let path: PathBuf = dir_entry.into_path();
                    // Silent skip - no glob match
                    if !include_set.is_match(&path) {
                        return None;
                    }
                    // Silent skip - exclusion
                    if exclude_set.is_match(&path) {
                        return None;
                    }

                    let path_identifier = match get_path_identifier(&path, &root) {
                        Ok(id) => id,
                        Err(e) => return Some(Err(WalkerError::PathIdentifierError(e))),
                    };

                    let fs_last_modified = match get_file_last_modified_date(&path) {
                        Ok(t) => t,
                        Err(e) => return Some(Err(WalkerError::ModificationDateReadError(e))),
                    };

                    let old_entry = old_snapshot
                        .get_entry_for_file_in_root(root.name(), path_identifier)
                        .cloned();
                    if let Some(entry) = old_entry
                        && fs_last_modified <= entry.last_modified()
                    {
                        let coppied_entry = SnapshotDelta {
                            root_id: root.name().to_string(),
                            path_identifier: path_identifier.to_string(),
                            entry,
                        };
                        return Some(Ok(BackupJob::AddCoppiedEntry(coppied_entry)));
                    }

                    Some(Ok(BackupJob::ProcessFileFurther {
                        path,
                        root: root.clone(),
                    }))
                }),
        )
    })
}

#[derive(Debug)]
enum BackupEvent {
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

pub fn run(config: Config) -> CLIResult<()> {
    // make a new snapshot
    let old_snapshot =
        Arc::new(Snapshot::load_latest_latest_snapshot(&config).map_err(CLIError::SnapshotError)?);

    let store = StoreBuilder::new(config.clone())
        .init_and_build()
        .map_err(CLIError::StoreError)?;

    let cores = available_parallelism().map(|x| x.get()).unwrap_or(1);

    let worker_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(cores)
        .build()
        .expect("Could not build worker pool");

    let (event_tx, event_rx) = crossbeam::channel::unbounded::<BackupEvent>();
    let (snapshot_tx, snapshot_rx) = crossbeam::channel::unbounded::<SnapshotDelta>();

    let config_clone = config.clone();
    let snapshot = thread::scope(move |s| {
        let walker_thread = s.spawn(move |_| {
            // FS Walker thread
            // iterate over all files to backup
            for job in root_walker(&config_clone, &old_snapshot) {
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
                                        event_tx_clone
                                            .send(BackupEvent::FinishBackup { path, root });
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
        });

        // Monitor thread
        let monitor_thread = s.spawn(move |_| {
            while let Ok(event) = event_rx.recv() {
                println!("{event:?}");
            }
        });

        // Snapshot builder thread
        let snapshot_builder = s.spawn(move |_| {
            let mut snapshot = Snapshot::empty();
            while let Ok(SnapshotDelta {
                root_id,
                path_identifier,
                entry,
            }) = snapshot_rx.recv()
            {
                snapshot
                    .get_root_entries_mut(&root_id)
                    .insert(path_identifier, entry);
            }
            snapshot
        });

        walker_thread.join();
        monitor_thread.join();

        snapshot_builder.join().expect("Join must succeed")
    })
    .expect("Scope cannot fail");
    Snapshot::save_snaphot(&snapshot, &config).map_err(CLIError::SnapshotError)?;

    Ok(())
}
