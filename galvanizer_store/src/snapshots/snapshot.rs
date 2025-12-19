use crate::snapshots::shapshot_entry::SnapshotEntry;
use galvanizer_config::config::ConfigError;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug, io};

pub type RootSnapshotEntries = HashMap<String, SnapshotEntry>;

/// Represents all saved information for all roots. Contains a map that assigns a map of snapshot entries to each root and their file id.
#[derive(Deserialize, Serialize)]
pub struct Snapshot {
    /// The key is the root id and it points to a map of snapshot entries, where the key is the relative path and the value is a SnapshotEntry.
    entries: HashMap<String, RootSnapshotEntries>,
}

impl Debug for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("snapshot");
        for (root_id, entries) in &self.entries {
            for (file_id, entry) in entries {
                s.field(&format!("{root_id} - {file_id}"), entry);
            }
        }
        s.finish()
    }
}

impl Snapshot {
    pub fn empty() -> Snapshot {
        Snapshot::new(HashMap::new())
    }

    pub fn new(entries: HashMap<String, RootSnapshotEntries>) -> Self {
        Self { entries }
    }

    pub fn get_entry_for_file_in_root(
        &self,
        root_id: &str,
        file_id: &str,
    ) -> Option<&SnapshotEntry> {
        self.entries.get(root_id)?.get(file_id)
    }

    pub fn get_root_entries_mut(&mut self, root_id: &str) -> &mut RootSnapshotEntries {
        self.entries.entry(root_id.to_string()).or_default()
    }

    pub fn get_root_ids(&self) -> impl Iterator<Item = &String> {
        self.entries.keys().into_iter()
    }

    pub fn get_rel_paths_for_root(&self, root_id: &str) -> Option<impl Iterator<Item = &String>> {
        Some(self.entries.get(root_id)?.keys().into_iter())
    }
}

#[derive(Debug)]
pub enum SnapshotError {
    SnapshotIOError(io::Error),
    SnapshotEntryReadError,
    InvalidSnapshotName(chrono::ParseError),
    SnapshotSerializationError(toml::ser::Error),
    SnapshotDeserializationError(toml::de::Error),
    ConfigError(ConfigError),
}
