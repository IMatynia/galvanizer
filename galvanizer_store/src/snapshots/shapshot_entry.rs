use std::fmt::Debug;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnapshotEntry {
    data_hash: String,
    last_modified: DateTime<Utc>,
}

impl SnapshotEntry {
    pub fn new(data_hash: String, last_modified: DateTime<Utc>) -> Self {
        Self {
            data_hash,
            last_modified,
        }
    }

    pub fn data_hash(&self) -> &str {
        &self.data_hash
    }

    pub fn last_modified(&self) -> DateTime<Utc> {
        self.last_modified
    }
}
