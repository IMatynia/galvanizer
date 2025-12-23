use std::{sync::Arc, thread::available_parallelism};

use crossbeam::channel;
use galvanizer_config::{Config, root_definition::RootDefinition};
use galvanizer_store::{
    snapshots::snapshot::Snapshot,
    store_cache_data_handler::{CacheHandlerError, uncompress_and_restore},
    store_cache_tools::original_file_path_from_identifier,
};
use log::{debug, error, trace};

use crate::{
    cli_errors::{CLIError, CLIResult},
    commands::QUEUE_SIZE,
    schemas::restore::RestoreArgs,
};

#[derive(Debug)]
pub enum RestoreErr {
    SkipBecauseCannotOverwrite,
    SkipBecauseCannotDeleteFiles, // TODO: split this into events
    SkipBecauseMissingFile,
    StoreEntryNotFound,
    CacheHandlerError(CacheHandlerError),
}

fn restore_a_file(
    snapshot: &Snapshot,
    parent_root: &RootDefinition,
    file_id: &str,
    restoration_options: &RestoreArgs,
    config: &Config,
) -> Result<(), RestoreErr> {
    

    debug!("Checking file {destination:?} for restoration from hash {hash_str}");



    debug!("Restoring file {destination:?}");
    uncompress_and_restore(config, &destination, &hash_str)
        .map_err(RestoreErr::CacheHandlerError)?;
    Ok(())
}

pub fn run(config: Config, restoration_options: RestoreArgs) -> CLIResult<()> {
    if restoration_options.delete_new_files {
        todo!("Currently the delete-new-files option is not available!");
    }

    let restoration_options = Arc::new(restoration_options);

    // Load the snapshot
    let snapshot = Snapshot::load_by_id_or_latest(&config, restoration_options.snapshot_id.clone())
        .map_err(CLIError::SnapshotError)?;
    let snapshot_arc = Arc::new(snapshot);

    let cores = available_parallelism().map(|x| x.get()).unwrap_or(1);
    rayon::scope(|s| {
        // dirwalker result channel
        let (tx, rx) = channel::bounded::<(String, RootDefinition)>(QUEUE_SIZE);

        // spawn worker processes
        for n in 0..cores {
            let rx = rx.clone();
            let snapshot = snapshot_arc.clone();
            let restoration_options = restoration_options.clone();
            let config = config.clone();
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
                        &config,
                    ) {
                        match e {
                            RestoreErr::StoreEntryNotFound => error!("No store entry found!"),
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
                    if tx.send((file_id.to_owned(), parent_root.clone())).is_err() {
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
