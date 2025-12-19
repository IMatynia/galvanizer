use galvanizer_config::{Config, root_definition::RootDefinition};
use galvanizer_store::{
    snapshots::snapshot::Snapshot, store::Store,
    store_cache_tools::original_file_path_from_identifier, store_error::StoreError,
};

use crate::{
    cli::restore::RestoreArgs,
    cli_errors::{CLIError, CLIResult},
};

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

    if file_currently_exists && restoration_options.dont_overwrite_files {
        return Err(RestoreErr::SkipBecauseCannotOverwrite);
    }

    if !file_currently_exists && restoration_options.skip_missing_file_restore {
        return Err(RestoreErr::SkipBecauseMissingFile);
    }

    if restoration_options.delete_new_files {
        todo!("Option delete new files is not implemented!");
    }

    store
        .directly_restore_file(&hash_str, &destination)
        .map_err(RestoreErr::OtherError)?;
    Ok(())
}

pub fn run(config: Config, restoration_options: RestoreArgs) -> CLIResult<()> {
    if restoration_options.delete_new_files {
        todo!("Currently the delete-new-files option is not available!");
    }

    // Load the snapshot
    let snapshot = {
        if let Some(snapshot_id) = restoration_options.snapshot_id {
            Snapshot::load_snapshot_by_id(&config, snapshot_id)
        } else {
            Snapshot::load_latest_latest_snapshot(&config)
        }
    }
    .map_err(CLIError::SnapshotError)?;

    let store = Store::new_uninitialized_store(config.clone())
        .initialize_hash_cache()
        .map_err(CLIError::StoreError)?;

    // Filter what files to restore
    for root in snapshot.get_root_ids() {
        if let Some(root_filter) = &restoration_options.root_id {
            if !(root_filter == root) {
                continue;
            }
        }
    }

    Ok(())
}
