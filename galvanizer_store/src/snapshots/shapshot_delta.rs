use galvanizer_config::root_definition::RootDefinition;

use crate::snapshots::shapshot_entry::SnapshotEntry;

pub struct SnapshotDelta {
    pub root: RootDefinition,
    pub file_id: String,
    pub entry: SnapshotEntry,
}
