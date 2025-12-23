use std::sync::Arc;

use crate::{
    cli_errors::{CLIError, CLIResult},
    commands::backup::{
        backup_monitor::{BackupEvent, backup_monitor_task},
        root_walker_thread::walker_thread_task,
        snapshot_builder::snapshot_builder_task,
    },
};
use crossbeam::thread;
use galvanizer_config::Config;
use galvanizer_store::{
    snapshots::{shapshot_delta::SnapshotDelta, snapshot::Snapshot},
    store_cache::hash_db::HashDB,
};

pub fn backup_command(config: Config) -> CLIResult<()> {
    let old_snapshot =
        Snapshot::load_latest_latest_snapshot(&config).map_err(CLIError::SnapshotError)?;

    let (event_tx, event_rx) = crossbeam::channel::unbounded::<BackupEvent>();
    let (snapshot_tx, snapshot_rx) = crossbeam::channel::unbounded::<SnapshotDelta>();

    let data_store_cache = Arc::new(
        HashDB::empty()
            .update_hash_set_with_cache_contents(
                &config
                    .get_store_cache_path()
                    .map_err(CLIError::ConfigError)?,
            )
            .map_err(CLIError::StoreError)?,
    );

    let config_clone = config.clone();
    let backup_scope = thread::scope(move |s| {
        let walker_thread = s.spawn(move |_| {
            walker_thread_task(
                config_clone,
                &old_snapshot,
                data_store_cache,
                event_tx,
                snapshot_tx,
            )
        });
        let monitor_thread = s.spawn(move |_| backup_monitor_task(event_rx));
        let snapshot_builder_thread = s.spawn(move |_| snapshot_builder_task(snapshot_rx));

        let _ = walker_thread.join();
        let _ = monitor_thread.join();
        snapshot_builder_thread.join()
    });

    let snapshot = backup_scope
        .map_err(|_| CLIError::TheadJoinError)?
        .map_err(|_| CLIError::TheadJoinError)?;

    Snapshot::save_snaphot(&snapshot, &config).map_err(CLIError::SnapshotError)?;

    Ok(())
}
