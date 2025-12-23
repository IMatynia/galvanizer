use crate::snapshots::shapshot_entry::SnapshotEntry;
use galvanizer_config::{config::ConfigError, root_definition::RootDefinition};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug, io};

#[derive(Deserialize, Serialize)]
pub struct RootSnapshotEntries {
    entries: HashMap<String, SnapshotEntry>,
    root_definition: RootDefinition,
}

impl RootSnapshotEntries {
    pub fn entries(&self) -> &HashMap<String, SnapshotEntry> {
        &self.entries
    }

    pub fn entries_mut(&mut self) -> &mut HashMap<String, SnapshotEntry> {
        &mut self.entries
    }

    pub fn root_definition(&self) -> &RootDefinition {
        &self.root_definition
    }
}

/// Represents all saved information for all roots. Contains a map that assigns a map of snapshot entries to each root and their file id.
#[derive(Deserialize, Serialize)]
pub struct Snapshot {
    /// The key is the root id and it points to a map of snapshot entries, where the key is the relative path and the value is a SnapshotEntry.
    roots: HashMap<String, RootSnapshotEntries>,
}

impl Debug for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("snapshot");
        for (root_id, entries) in &self.roots {
            for (file_id, entry) in entries.entries() {
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
        Self { roots: entries }
    }

    pub fn get_entry_for_file_in_root(
        &self,
        root_id: &str,
        file_id: &str,
    ) -> Option<&SnapshotEntry> {
        self.roots.get(root_id)?.entries().get(file_id)
    }

    pub fn get_root_entries_mut(
        &mut self,
        root_definition: RootDefinition,
    ) -> &mut RootSnapshotEntries {
        self.roots
            .entry(root_definition.name().to_string())
            .or_insert(RootSnapshotEntries {
                entries: HashMap::new(),
                root_definition,
            })
    }

    pub fn get_roots(&self) -> &HashMap<String, RootSnapshotEntries> {
        &self.roots
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

pub type SnapshotResult<T> = Result<T, SnapshotError>;
