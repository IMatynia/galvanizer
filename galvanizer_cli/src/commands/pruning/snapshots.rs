use galvanizer_config::Config;
use galvanizer_store::snapshots::snapshot::Snapshot;

use crate::cli_errors::{CLIError, CLIResult};

/// Sort snapshots alphabetically and skip n off the top.
pub fn get_snapshots_to_prune_and_leave_n_newest(
    config: &Config,
    n: usize,
) -> CLIResult<Vec<String>> {
    let mut all_snapshot_ids: Vec<String> = Snapshot::load_all_snapshot_ids(config)
        .map_err(CLIError::SnapshotError)?
        .collect();

    all_snapshot_ids.sort();
    Ok(all_snapshot_ids.into_iter().skip(n).collect())
}
