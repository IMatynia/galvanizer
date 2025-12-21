use galvanizer_config::Config;
use galvanizer_store::{
    snapshots::{shapshot_delta::SnapshotDelta, snapshot::Snapshot},
    store::StoreBuilder,
};
use log::error;
use crate::{
    cli_errors::{CLIError, CLIResult},
    commands::backup::{
        backup_command_schemas::BackupEvent, backup_monitor::backup_monitor_task,
        snapshot_builder::snapshot_builder_task, walker_thread::walker_thread_task,
    },
};
use crossbeam::thread;

pub fn backup_command(config: Config) -> CLIResult<()> {
    let old_snapshot =
        Snapshot::load_latest_latest_snapshot(&config).map_err(CLIError::SnapshotError)?;

    let store = StoreBuilder::new(config.clone())
        .init_and_build()
        .map_err(CLIError::StoreError)?;

    let (event_tx, event_rx) = crossbeam::channel::unbounded::<BackupEvent>();
    let (snapshot_tx, snapshot_rx) = crossbeam::channel::unbounded::<SnapshotDelta>();

    let config_clone = config.clone();
    let backup_scope = thread::scope(move |s| {
        let walker_thread = s.spawn(move |_| {
            walker_thread_task(config_clone, &old_snapshot, store, event_tx, snapshot_tx)
        });
        let monitor_thread = s.spawn(move |_| backup_monitor_task(event_rx));
        let snapshot_builder_thread = s.spawn(move |_| snapshot_builder_task(snapshot_rx));

        if let Err(e) = walker_thread.join() {
            error!("Failed to join walker thread: {e:?}");
        }
        if let Err(e) = monitor_thread.join() {
            error!("Failed to join monitor thread: {e:?}");
        }
        snapshot_builder_thread.join()
    });

    let snapshot = backup_scope
        .map_err(|_| CLIError::TheadJoinError)?
        .map_err(|_| CLIError::TheadJoinError)?;

    Snapshot::save_snaphot(&snapshot, &config).map_err(CLIError::SnapshotError)?;

    Ok(())
}
