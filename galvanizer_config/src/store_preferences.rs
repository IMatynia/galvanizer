use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub const DEFAULT_PREFIX_LENGTH: u16 = 2;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StorePreferences {
    path: PathBuf,
    subfolder_prefix_length: Option<u16>,
}

impl StorePreferences {
    pub fn new(path: PathBuf, subfolder_prefix_length: Option<u16>) -> Self {
        Self {
            path,
            subfolder_prefix_length,
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn subfolder_prefix_length(&self) -> u16 {
        self.subfolder_prefix_length
            .unwrap_or(DEFAULT_PREFIX_LENGTH)
    }
}
