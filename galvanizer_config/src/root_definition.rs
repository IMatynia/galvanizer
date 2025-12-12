use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
}
