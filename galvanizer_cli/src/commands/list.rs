use std::{fs::read_dir, io};

use crate::{
    cli_errors::{CLIError, CLIResult},
    schemas::list::ListCommands,
};
use galvanizer_config::Config;
use galvanizer_store::snapshots::snapshot::{Snapshot, SnapshotError};

fn snapshot_io_error(e: io::Error) -> CLIError {
    CLIError::SnapshotError(SnapshotError::SnapshotIOError(e))
}

pub fn run(config: Config, command_options: ListCommands) -> CLIResult<()> {
    match command_options {
        ListCommands::Snapshots => {
            let snapshots_dir = config.get_snaphots_path().map_err(CLIError::ConfigError)?;
            println!("All stored snapshots in {}:", snapshots_dir.display());
            for snapshot_path in read_dir(snapshots_dir)
                .map_err(snapshot_io_error)?
                .map(|d| d.map_err(snapshot_io_error))
            {
                match snapshot_path {
                    Ok(entry) => {
                        if let Some(stem) = entry.path().file_stem() {
                            println!("{}", stem.display());
                        }
                    }
                    Err(e) => eprintln!("{e:?}"),
                }
            }
        }
        ListCommands::Roots { snapshot_id } => {
            let snapshot = Snapshot::load_by_id_or_latest(&config, snapshot_id.clone())
                .map_err(CLIError::SnapshotError)?;

            println!(
                "All roots for snapshot {}:",
                snapshot_id.unwrap_or("LATEST".into())
            );
            for root in snapshot.get_root_ids() {
                println!("{root}");
            }
        }
        ListCommands::Files {
            snapshot_id,
            root_id,
        } => {
            let snapshot = Snapshot::load_by_id_or_latest(&config, snapshot_id.clone())
                .map_err(CLIError::SnapshotError)?;

            println!(
                "All files in root {root_id} for snapshot {}:",
                snapshot_id.unwrap_or("LATEST".into())
            );
            if let Some(files) = snapshot.get_rel_paths_for_root(&root_id) {
                for file_id in files {
                    println!("{file_id}");
                }
            } else {
                println!("Root not found! Maybe you made a typo?");
            }
        }
    }

    Ok(())
}
