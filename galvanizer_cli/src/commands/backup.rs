use galvanizer_config::{Config, root_definition::RootDefinition};
use galvanizer_store::{snapshots::snapshot::Snapshot, store::Store};
use log::{error, trace};

use crate::cli_errors::{CLIError, CLIResult};
use crossbeam::channel::{self, Sender};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    thread::available_parallelism,
};
use walkdir::WalkDir;

fn handle_root(
    config: &Config,
    root: &RootDefinition,
    tx: &Sender<(PathBuf, RootDefinition)>,
) -> CLIResult<()> {
    let root_prefs = config
        .get_combined_preferences_for_root(root.name())
        .expect("Looked up non existent root id!");
    let include_set = root_prefs
        .inclusions_globset()
        .map_err(CLIError::ConfigError)?;
    let exclude_set = root_prefs
        .exclusions_globset()
        .map_err(CLIError::ConfigError)?;

    // for every entry in the directories
    for entry in WalkDir::new(root.path()).into_iter() {
        let entry = entry.map_err(CLIError::DirWalkError)?;
        trace!("Checking file: {entry:?}");
        if !entry.file_type().is_file() {
            trace!("Path is not a file, skipping!");
            continue;
        }

        let path = entry.path();

        // Check include first
        if !include_set.is_match(path) {
            trace!("Path does not match include set!");
            continue;
        }

        // Check exclusion
        if exclude_set.is_match(path) {
            trace!("Path matches exclude set!");
            continue;
        }

        // Send to workers
        // If channel is full, this blocks until workers consume more
        trace!("Sending file {path:?} to be processed!");
        tx.send((path.to_path_buf(), root.clone())).unwrap();
    }
    Ok(())
}

pub fn run(config: Config) -> CLIResult<()> {
    // make a new snapshot
    let snapshot =
        Snapshot::load_latest_latest_snapshot(&config).map_err(CLIError::SnapshotError)?;
    let snaphot_arc = Arc::new(Mutex::new(snapshot));

    let store = Store::new_uninitialized_store(config.clone())
        .initialize_hash_cache()
        .map_err(CLIError::StoreError)?;
    let cores = available_parallelism().map(|x| x.get()).unwrap_or(1);
    rayon::scope(|s| {
        // dirwalker result channel
        let (tx, rx) = channel::bounded::<(PathBuf, RootDefinition)>(1024);

        // spawn worker processes
        for n in 0..cores {
            let rx = rx.clone();
            let mut store = store.clone();
            let snapshot = snaphot_arc.clone();
            s.spawn(move |_| {
                while let Ok((path, root)) = rx.recv() {
                    trace!("Worker {n} handling file {path:?}");
                    if let Err(e) = store.store_file(&path, &root, &snapshot) {
                        eprint!("W{n} => Store error: {e:?}");
                    }
                }
            });
        }

        // iterate over all backup roots, walk them dirs
        for root in config.backup_roots() {
            if let Err(e) = handle_root(&config, root, &tx) {
                error!("An error occured while loading files from root: {e:?}")
            }
        }
    });

    // save the new snapshot
    let snapshot = snaphot_arc.lock().unwrap();
    Snapshot::save_snaphot(&snapshot, &config).map_err(CLIError::SnapshotError)?;
    Ok(())
}
