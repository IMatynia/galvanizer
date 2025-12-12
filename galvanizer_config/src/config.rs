use std::{fs, io, path::PathBuf};

use crate::{
    compression::CompressionOptions, root_definition::RootDefinition,
    root_preferences::RootPreferences, store_preferences::StorePreferences,
};
use serde::{Deserialize, Serialize};

const DATA_STORE_FOLDER: &str = "data_store";
const SNAPSHOTS_FOLDER: &str = "snapshots";
const CACHE_FILE_EXTENSION: &str = "bin";
pub const CURRENT_VERISON: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    version: String,
    store: StorePreferences,
    compression_settings: Option<CompressionOptions>,
    default_root_preferences: Option<RootPreferences>,
    backup_roots: Vec<RootDefinition>,
}

#[derive(Debug)]
pub enum ConfigError {
    NoStoreCachePathAvailabile,
    FailedToCreateFolder(io::Error),
    PatternError(globset::Error),
}

pub type ConfigResult<T> = Result<T, ConfigError>;

impl Config {
    pub fn new(
        version: String,
        store: StorePreferences,
        compression_settings: Option<CompressionOptions>,
        default_root_preferences: Option<RootPreferences>,
        backup_roots: Vec<RootDefinition>,
    ) -> ConfigResult<Self> {
        let config = Self {
            version,
            store,
            compression_settings,
            default_root_preferences,
            backup_roots,
        };

        let path = config.get_store_cache_path()?;
        fs::DirBuilder::new()
            .recursive(true)
            .create(path)
            .map_err(ConfigError::FailedToCreateFolder)?;

        let path = config.get_snaphots_path()?;
        fs::DirBuilder::new()
            .recursive(true)
            .create(path)
            .map_err(ConfigError::FailedToCreateFolder)?;

        Ok(config)
    }

    pub fn get_root(&self) -> PathBuf {
        self.store().path().clone()
    }

    pub fn get_store_cache_path(&self) -> ConfigResult<PathBuf> {
        let mut p = self.get_root();
        p.push(DATA_STORE_FOLDER);
        fs::DirBuilder::new()
            .recursive(true)
            .create(&p)
            .map_err(ConfigError::FailedToCreateFolder)?;
        Ok(p)
    }

    pub fn get_snaphots_path(&self) -> ConfigResult<PathBuf> {
        let mut p = self.get_root();
        p.push(SNAPSHOTS_FOLDER);
        fs::DirBuilder::new()
            .recursive(true)
            .create(&p)
            .map_err(ConfigError::FailedToCreateFolder)?;
        Ok(p)
    }

    pub fn get_store_cache_path_for_hash(&self, hash_str: &str) -> ConfigResult<PathBuf> {
        let prefix_length: usize = self.store().subfolder_prefix_length().into();
        let prefix: String = hash_str.chars().take(prefix_length).collect();
        let path = self
            .get_store_cache_path()?
            .join(prefix)
            .join(hash_str)
            .with_extension(CACHE_FILE_EXTENSION);
        fs::DirBuilder::new()
            .recursive(true)
            .create(&path)
            .map_err(ConfigError::FailedToCreateFolder)?;
        Ok(path)
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn compression_settings(&self) -> Option<&CompressionOptions> {
        self.compression_settings.as_ref()
    }

    pub fn default_root_preferences(&self) -> Option<&RootPreferences> {
        self.default_root_preferences.as_ref()
    }

    pub fn store(&self) -> &StorePreferences {
        &self.store
    }

    pub fn backup_roots(&self) -> &[RootDefinition] {
        &self.backup_roots
    }

    pub fn find_root_by_id(&self, id: &str) -> Option<&RootDefinition> {
        self.backup_roots().iter().find(|r| r.name() == id)
    }

    pub fn get_combined_preferences_for_root(&self, id: &str) -> Option<RootPreferences> {
        let root = self.find_root_by_id(id)?;
        let root_prefs = root.preferences();

        Some(RootPreferences::fill_with_priority(
            root_prefs.cloned(),
            self.default_root_preferences.clone(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Config, compression::CompressionOptions, root_definition::RootDefinition,
        store_preferences::StorePreferences, tests::resources,
    };

    fn mock_config() -> Config {
        let version = "1.2.3".into();
        let store = StorePreferences::new(resources().join("example_store"), None);
        let compression_settings = Some(CompressionOptions::Snap);
        let root = RootDefinition::new(
            "example_root".into(),
            resources().join("example_files"),
            None,
        );
        Config::new(
            version,
            store,
            compression_settings,
            None,
            vec![root.clone()],
        )
        .unwrap()
    }

    #[test]
    fn test_config_init() {
        let config = mock_config();

        assert!(
            config
                .get_store_cache_path_for_hash("0123456789ABCDEF")
                .unwrap()
                .to_string_lossy()
                .ends_with("/data_store/01/0123456789ABCDEF.bin"),
        );

        assert_eq!(config.store().subfolder_prefix_length(), 2);

        assert_eq!(config.backup_roots().len(), 1);

        assert_eq!(
            config.find_root_by_id("example_root").unwrap().name(),
            "example_root"
        );

        assert_eq!(
            config
                .get_combined_preferences_for_root("example_root")
                .unwrap()
                .exclusions(),
            None
        );
    }
}
