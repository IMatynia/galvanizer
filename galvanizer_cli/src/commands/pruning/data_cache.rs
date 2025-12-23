use galvanizer_config::Config;
use galvanizer_store::{
    snapshot_walker::{self},
    snapshots::snapshot::Snapshot,
    store_cache::hash_db::HashDB,
};

use crate::cli_errors::{CLIError, CLIResult};

pub fn get_unused_hashes_to_prune(config: &Config) -> CLIResult<Vec<String>> {
    let stored_hashes = HashDB::empty()
        .update_hash_set_with_cache_contents(
            &config
                .get_store_cache_path()
                .map_err(CLIError::ConfigError)?,
        )
        .map_err(CLIError::StoreError)?;

    let used_hashes = HashDB::empty();

    for snapshot_id in Snapshot::load_all_snapshot_ids(config).map_err(CLIError::SnapshotError)? {
        let snapshot = Snapshot::load_by_id_or_latest(config, Some(snapshot_id))
            .map_err(CLIError::SnapshotError)?;
        for entry in snapshot_walker::walk_all_files_in_snapshot(&snapshot) {
            used_hashes.insert(entry.hash_str);
        }
    }

    Ok(stored_hashes
        .into_hashset()
        .difference(&used_hashes.into_hashset())
        .map(|x| x.to_owned())
        .collect())
}
