use crate::snapshots::shapshot_entry::SnapshotEntry;

pub struct SnapshotDelta {
    pub root_id: String,
    pub path_identifier: String,
    pub entry: SnapshotEntry,
}
