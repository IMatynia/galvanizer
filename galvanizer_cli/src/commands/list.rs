use crate::{
    cli_errors::{CLIError, CLIResult},
    schemas::list::ListCommands,
};
use galvanizer_config::Config;
use galvanizer_store::snapshots::snapshot::Snapshot;

pub fn run(config: Config, command_options: ListCommands) -> CLIResult<()> {
    match command_options {
        ListCommands::Snapshots => {
            let snapshots_dir = config.get_snaphots_path().map_err(CLIError::ConfigError)?;
            println!("All stored snapshots in {}:", snapshots_dir.display());
            for snapshot_id in
                Snapshot::load_all_snapshot_ids(&config).map_err(CLIError::SnapshotError)?
            {
                println!("{}", snapshot_id);
            }
        }
        ListCommands::Roots { snapshot_id } => {
            let snapshot = Snapshot::load_by_id_or_latest(&config, snapshot_id.clone())
                .map_err(CLIError::SnapshotError)?;

            println!(
                "All roots for snapshot {}:",
                snapshot_id.unwrap_or("LATEST".into())
            );
            for root in snapshot.get_roots().keys() {
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
            if let Some(files) = snapshot
                .get_roots()
                .get(&root_id)
                .map(|root_entry| root_entry.entries().keys())
            {
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
