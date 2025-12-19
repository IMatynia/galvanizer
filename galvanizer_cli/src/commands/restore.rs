use std::{sync::Arc, thread::available_parallelism};

use crossbeam::channel;
use galvanizer_config::{Config, root_definition::RootDefinition};
use galvanizer_store::{
    snapshots::snapshot::Snapshot, store::Store,
    store_cache_tools::original_file_path_from_identifier, store_error::StoreError,
};
use log::{debug, error, trace};

use crate::{
    cli::restore::RestoreArgs,
    cli_errors::{CLIError, CLIResult},
};

#[derive(Debug)]
pub enum RestoreErr {
    SkipBecauseCannotOverwrite,
    SkipBecauseCannotDeleteFiles,
    SkipBecauseMissingFile,
    StoreEntryNotFound,
    OtherError(StoreError),
}

fn restore_a_file(
    snapshot: &Snapshot,
    parent_root: &RootDefinition,
    file_id: &str,
    restoration_options: &RestoreArgs,
    store: &Store,
) -> Result<(), RestoreErr> {
    let hash_str = snapshot
        .get_entry_for_file_in_root(parent_root.name(), file_id)
        .ok_or(RestoreErr::StoreEntryNotFound)?
        .data_hash()
        .to_string();

    let destination = original_file_path_from_identifier(file_id, parent_root);
    let file_currently_exists = destination.exists();

    debug!("Checking file {destination:?} for restoration from hash {hash_str}");

    if file_currently_exists && restoration_options.dont_overwrite_files {
        return Err(RestoreErr::SkipBecauseCannotOverwrite);
    }

    if !file_currently_exists && restoration_options.skip_missing_file_restore {
        return Err(RestoreErr::SkipBecauseMissingFile);
    }

    if restoration_options.delete_new_files {
        todo!("Option delete new files is not implemented!");
    }

    debug!("Restoring file {destination:?}");
    store
        .directly_restore_file(&hash_str, &destination)
        .map_err(RestoreErr::OtherError)?;
    Ok(())
}

pub fn run(config: Config, restoration_options: RestoreArgs) -> CLIResult<()> {
    if restoration_options.delete_new_files {
        todo!("Currently the delete-new-files option is not available!");
    }

    let restoration_options = Arc::new(restoration_options);

    // Load the snapshot
    let snapshot = {
        if let Some(snapshot_id) = &restoration_options.snapshot_id {
            debug!("Loading snapshot {snapshot_id}");
            Snapshot::load_snapshot_by_id(&config, snapshot_id.into())
        } else {
            debug!("Loading latest snapshot");
            Snapshot::load_latest_latest_snapshot(&config)
        }
    }
    .map_err(CLIError::SnapshotError)?;
    let snapshot_arc = Arc::new(snapshot);

    debug!("Initializing store");
    let store = Store::new_uninitialized_store(config.clone())
        .initialize_hash_cache()
        .map_err(CLIError::StoreError)?;

    let cores = available_parallelism().map(|x| x.get()).unwrap_or(1);
    rayon::scope(|s| {
        // dirwalker result channel
        let (tx, rx) = channel::bounded::<(String, RootDefinition)>(1024);

        // spawn worker processes
        for n in 0..cores {
            let rx = rx.clone();
            let store = store.clone();
            let snapshot = snapshot_arc.clone();
            let restoration_options = restoration_options.clone();
            s.spawn(move |_| {
                while let Ok((file_id, parent_root)) = rx.recv() {
                    trace!(
                        "Worker {n} handling file {} of {}",
                        file_id,
                        parent_root.name()
                    );
                    if let Err(e) = restore_a_file(
                        &snapshot,
                        &parent_root,
                        &file_id,
                        &restoration_options,
                        &store,
                    ) {
                        match e {
                            RestoreErr::StoreEntryNotFound => error!("No store entry found!"),
                            RestoreErr::OtherError(store_error) => {
                                error!("Error during restoration: {store_error:?}")
                            }
                            _ => debug!("{e:?}"),
                        }
                    }
                }
            });
        }

        for parent_root in snapshot_arc
            .get_root_ids()
            .filter_map(|root_id| config.find_root_by_id(root_id))
        {
            if let Some(root_filter) = &restoration_options.root_id
                && (root_filter != parent_root.name())
            {
                continue;
            }

            debug!("Restoring root {}", parent_root.name());

            if let Some(files) = snapshot_arc.get_rel_paths_for_root(parent_root.name()) {
                for file_id in files {
                    if let Err(_) = tx.send((file_id.to_owned(), parent_root.clone())) {
                        error!("An error occured on sending tasks to workers!");
                    }
                }
            } else {
                error!("Invalid root found in config: {}", parent_root.name());
            }
        }
    });

    Ok(())
}
