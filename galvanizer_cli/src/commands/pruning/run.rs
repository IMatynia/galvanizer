use std::{fs, path::PathBuf};

use galvanizer_config::Config;
use galvanizer_store::snapshots::snapshot::Snapshot;

use crate::{
    cli_errors::{CLIError, CLIResult},
    commands::pruning::{
        data_cache::get_unused_hashes_to_prune,
        snapshots::get_snapshots_to_prune_and_leave_n_newest,
    },
    schemas::prune::{PruneArgs, PruneCommands},
};

pub fn prune_command(config: Config, args: PruneArgs) -> CLIResult<()> {
    let files_to_delete: Vec<PathBuf> = match args.command {
        PruneCommands::UnusedData => {
            let unused_hashes = get_unused_hashes_to_prune(&config)?;
            unused_hashes
                .into_iter()
                .filter_map(|hash| config.get_store_cache_path_for_hash(&hash).ok())
                .collect()
        }
        PruneCommands::KeepSnapshots { n } => {
            let old_snapshots = get_snapshots_to_prune_and_leave_n_newest(&config, n)?;
            old_snapshots
                .into_iter()
                .filter_map(|snapshot_id| {
                    Snapshot::get_snapshot_path_from_id(&config, snapshot_id).ok()
                })
                .collect()
        }
    };

    let bytes_to_delete = files_to_delete.iter().fold(0u64, |accumulated, file| {
        if let Ok(metadata) = file.metadata() {
            metadata.len() + accumulated
        } else {
            0 + accumulated
        }
    });

    let megabytes_to_delete = bytesize::ByteSize(bytes_to_delete).as_mb();

    if dialoguer::Confirm::new()
        .with_prompt(format!(
            "You are about to prune {} files ({:0.3} MB). Do you want to continue?", 
            files_to_delete.len(), megabytes_to_delete
        ))
        .interact()
        .expect("UI")
    {
        for file in files_to_delete {
            if let Err(e) = fs::remove_file(file) {
                eprintln!("Error: {e:?}");
            }
        }
    }

    Ok(())
}
