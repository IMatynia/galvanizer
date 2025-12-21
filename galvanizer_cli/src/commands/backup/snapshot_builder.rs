use crossbeam::channel::Receiver;
use galvanizer_store::snapshots::{shapshot_delta::SnapshotDelta, snapshot::Snapshot};

pub fn snapshot_builder_task(snapshot_rx: Receiver<SnapshotDelta>) -> Snapshot {
    let mut snapshot = Snapshot::empty();
    while let Ok(SnapshotDelta {
        root_id,
        path_identifier,
        entry,
    }) = snapshot_rx.recv()
    {
        snapshot
            .get_root_entries_mut(&root_id)
            .insert(path_identifier, entry);
    }
    snapshot
}
