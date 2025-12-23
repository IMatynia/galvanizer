/// # Galvanizer data store
///
/// ## Store directory structure
///
/// ```text
/// <store_root>
/// |
/// |-> data_store
/// |   |-> <base64 hash prefix>
/// |   |   |-> <base64 hash of uncompressed file as filename> -> compressed content of a file identified by the hash
/// |   |   |...
/// |   |-> <base64 hash prefix>
/// |   |...
/// |-> snapshots
///     |-> <snapshot date>.toml -> file structure of the given root. Contains a list of relative paths, names and file hashes for lookup
///     |-> <another snapshot>.toml
///     | ...   
/// ```
use std::{
    fs::{self, DirEntry},
    path::Path,
    sync::Arc,
};

use crate::{
    snapshots::{shapshot_delta::SnapshotDelta, shapshot_entry::SnapshotEntry},
    store_cache_data_handler::compress_and_store_file,
    store_cache_tools::{evaluate_file_sha512_hash, get_file_last_modified_date},
    store_error::{StoreError, StoreResult},
};
use dashmap::DashSet;
use galvanizer_config::{Config, root_definition::RootDefinition};
use log::debug;

/// Internally thread-safe handler for the data store. Allows you to store a file safely and reliably.
#[derive(Clone)]
pub struct Store {
    /// Primary config
    config: Config,

    /// Hashes of files already stored in the database
    data_store_cache: Arc<DashSet<String>>,
}

pub struct StoreBuilder(Config);

impl StoreBuilder {
    pub fn new(config: Config) -> StoreBuilder {
        StoreBuilder(config)
    }

    pub fn store_cache_file_iterator(
        store_root: &Path,
    ) -> StoreResult<impl Iterator<Item = DirEntry>> {
        Ok(fs::read_dir(store_root)
            .map_err(StoreError::CannotReadStoreCacheFolder)?
            .flatten()
            .flat_map(|x| fs::read_dir(x.path()))
            .flatten()
            .flatten())
    }

    fn update_hash_set_with_cache_contents(
        store_root: &Path,
        set: &DashSet<String>,
    ) -> StoreResult<()> {
        Self::store_cache_file_iterator(store_root)?
            .flat_map(|x| x.path().file_stem().map(|s| s.to_owned()))
            .map(|x| x.display().to_string())
            .fold(set, |set, hash| {
                set.insert(hash);
                set
            });
        Ok(())
    }
    /// Updates store data chache contents. Make sure to update BEFORE sending a clone to the threaded/parallel context
    pub fn init_and_build(self) -> StoreResult<Store> {
        // Read only valid entries in subdirectories
        let config = self.0;
        let dashset: DashSet<String> = DashSet::new();
        Self::update_hash_set_with_cache_contents(
            &config
                .get_store_cache_path()
                .map_err(StoreError::ConfigurationError)?,
            &dashset,
        )?;
        Ok(Store {
            config,
            data_store_cache: Arc::new(dashset),
        })
    }
}

impl Store {
    pub fn store_file(
        &self,
        path: &Path,
        parent_root: &RootDefinition,
    ) -> StoreResult<SnapshotDelta> {
        // Query for last modified date
        let fs_last_modified =
            get_file_last_modified_date(path).map_err(StoreError::FileMetadataError)?;

        // Get path identifier str
        let path_identifier = parent_root
            .make_path_identifier(path)
            .map_err(StoreError::PathIdentifierError)?;

        // Read file hash
        let hash_str = evaluate_file_sha512_hash(path).map_err(StoreError::ErrorDuringHashEval)?;
        if hash_str.is_empty() {
            return Err(StoreError::CriticalHashError);
        }

        // heavy lifting - exclusive access to data_store_cache for checking and updating contents
        if self.data_store_cache.insert(hash_str.clone()) {
            debug!("New hash found: {hash_str}\nCompressing and storing the file");
            // Compress and store the file
            compress_and_store_file(&self.config, path, &hash_str)
                .map_err(StoreError::CacheHandlerError)?;
        }

        Ok(SnapshotDelta {
            root_id: parent_root.name().to_string(),
            path_identifier: path_identifier.to_string(),
            entry: SnapshotEntry::new(hash_str, fs_last_modified),
        })
    }
}

#[cfg(test)]
mod tests {
    fn init() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Trace)
            .try_init();
    }
    use std::sync::{Arc, Mutex};

    use galvanizer_config::{
        Config, compression::CompressionOptions, root_definition::RootDefinition,
        store_preferences::StorePreferences,
    };

    use crate::{snapshots::snapshot::Snapshot, store::StoreBuilder, tests::resources};

    fn mock_root() -> RootDefinition {
        let test_backup = resources().join("example_files");
        RootDefinition::new("example_root".into(), test_backup, None)
    }

    fn mock_config() -> Config {
        let test_dir = resources().join("example_store");
        Config::new(
            "1.2.3".into(),
            StorePreferences::new(test_dir, None),
            Some(CompressionOptions::Snap),
            None,
            vec![mock_root()],
        )
        .expect("tests")
    }

    fn mock_store() -> StoreBuilder {
        let config = mock_config();
        StoreBuilder::new(config)
    }

    #[test]
    fn test_store() {
        init();
        let store = mock_store().init_and_build().unwrap();

        let file_path = resources()
            .join("example_files")
            .join("subfolder")
            .join("some_file.txt");
        let root = mock_root();
        let snapshot = Arc::new(Mutex::new(Snapshot::empty()));
        store.store_file(&file_path, &root).unwrap();
        store.store_file(&file_path, &root).unwrap();
        store.store_file(&file_path, &root).unwrap();

        println!("{:?}", snapshot.lock().unwrap());
    }
}
