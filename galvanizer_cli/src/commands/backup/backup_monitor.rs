use crossbeam::channel::Receiver;

use crate::commands::backup::backup_command_schemas::BackupEvent;

pub fn backup_monitor_task(event_rx: Receiver<BackupEvent>) {
    while let Ok(event) = event_rx.recv() {
        println!("{event:?}");
    }
}
