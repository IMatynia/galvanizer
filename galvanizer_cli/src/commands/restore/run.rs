use crate::{
    cli_errors::{CLIError, CLIResult},
    commands::restore::{
        restore_monitor::{RestoreEvent, restore_monitor_task},
        snapshot_walker_thread::snapshot_walker_task,
    },
};
use crossbeam::thread;
use galvanizer_config::Config;
use galvanizer_store::{snapshot_walker::RestoreArgs, snapshots::snapshot::Snapshot};

pub fn restore_command(config: Config, restoration_options: RestoreArgs) -> CLIResult<()> {
    if restoration_options.delete_new_files {
        todo!("Currently the delete-new-files option is not available!");
    }

    // Load the snapshot
    let snapshot = Snapshot::load_by_id_or_latest(&config, restoration_options.snapshot_id.clone())
        .map_err(CLIError::SnapshotError)?;

    let (event_tx, event_rx) = crossbeam::channel::unbounded::<RestoreEvent>();

    let _ = thread::scope(move |s| {
        let walker_thread = s.spawn(move |_| {
            snapshot_walker_task(config, event_tx, snapshot, restoration_options);
        });
        let monitor_thread = s.spawn(move |_| restore_monitor_task(event_rx));

        let _ = walker_thread.join();
        let _ = monitor_thread.join();
    });

    Ok(())
}
