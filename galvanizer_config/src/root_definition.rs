use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::root_preferences::RootPreferences;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RootDefinition {
    name: String,
    path: PathBuf,
    preferences: Option<RootPreferences>,
}

impl RootDefinition {
    pub fn new(name: String, path: PathBuf, preferences: Option<RootPreferences>) -> Self {
        Self {
            name,
            path,
            preferences,
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn preferences(&self) -> Option<&RootPreferences> {
        self.preferences.as_ref()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn make_path_identifier<'a>(&self, path: &'a Path) -> Result<&'a str, &'static str> {
        let path_short = path
            .strip_prefix(self.path())
            .map_err(|_| "File is not within the current root directory!")?;
        let path_identifier = path_short
            .as_os_str()
            .to_str()
            .ok_or("Failed to read path as utf-8 string")?;
        Ok(path_identifier)
    }

    pub fn make_original_file_path_from_identifier(&self, file_id: &str) -> PathBuf {
        self.path().clone().join(file_id)
    }
}
