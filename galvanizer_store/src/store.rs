/// # Galvanizer data store
///
/// ## Store directory structure
///
/// ```text
/// <store_root>
/// |
/// |-> data_store
/// |   |-> <hash prefix>
/// |   |   |-> <hash of uncompressed file as filename> -> compressed content of a file identified by the hash
/// |   |   |...
/// |   |-> <hash prefix>
/// |   |...
/// |-> snapshots
///     |-> <snapshot date>
///     |   |-> <root name>.toml -> file structure of the given root. Contains a list of relative paths, names and file hashes for lookup
///     |   |-> <other root name>.toml
///     |   |...
///     |-> <another snapshot>
///     | ...   
/// ```
use std::{
    collections::HashSet,
    fs::{self, DirEntry},
    mem::swap,
    path::Path,
    sync::{Arc, Mutex, MutexGuard},
};

use crate::{
    snapshots::{shapshot_entry::SnapshotEntry, snapshot::Snapshot},
    store_cache_data_handler::StoreCacheDataHandler,
    store_cache_tools::{
        evaluate_file_sha512_hash, get_file_last_modified_date, get_path_identifier,
        original_file_path_from_identifier,
    },
    store_error::{StoreError, StoreResult},
};
use galvanizer_config::{Config, root_definition::RootDefinition};
use log::debug;

#[derive(Clone)]
pub struct Store {
    /// Primary config
    config: Arc<Config>,

    /// Hashes of files already stored in the database
    data_store_cache: Arc<Mutex<HashSet<String>>>,

    /// Current snapshot updated in parallel
    current_snapshot: Arc<Mutex<Snapshot>>,

    /// Data handler
    cache_data_handler: Arc<StoreCacheDataHandler>,
}

pub struct UninitializedStore(Store);

impl UninitializedStore {
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
        set: &mut HashSet<String>,
    ) -> StoreResult<()> {
        Self::store_cache_file_iterator(store_root)?
            .flat_map(|x| x.path().file_stem().map(|s| s.to_owned()))
            .map(|x| x.to_string_lossy().to_string())
            .fold(set, |set, hash| {
                set.insert(hash);
                set
            });
        Ok(())
    }
    /// Updates store data chache contents. Make sure to update BEFORE sending a clone to the threaded/parallel context
    pub fn initialize_hash_cache(self) -> StoreResult<Store> {
        // Read only valid entries in subdirectories
        let store = self.0;
        {
            let mut data_store_cache_lock = store.data_store_cache.lock().unwrap();
            Self::update_hash_set_with_cache_contents(
                &store
                    .config
                    .get_store_cache_path()
                    .map_err(StoreError::ConfigurationError)?,
                &mut data_store_cache_lock,
            )?;
        }
        Ok(store)
    }
}

impl Store {
    pub fn new_uninitialized_store(
        config: Config,
        current_snapshot: Snapshot,
    ) -> UninitializedStore {
        let config_arc = Arc::new(config);
        let snaphot_arc = Arc::new(Mutex::new(current_snapshot));
        UninitializedStore(Store {
            config: Arc::clone(&config_arc),
            data_store_cache: Default::default(),
            current_snapshot: snaphot_arc,
            cache_data_handler: Arc::new(StoreCacheDataHandler::new(Arc::clone(&config_arc))),
        })
    }

    pub fn store_file(&mut self, path: &Path, parent_root: &RootDefinition) -> StoreResult<()> {
        // Query for last modified date
        let fs_last_modified =
            get_file_last_modified_date(path).map_err(StoreError::FileMetadataError)?;

        // Get path identifier str
        let path_identifier = get_path_identifier(path, parent_root)?;

        // get associated entry in the current snapshot
        // check if the file was modified since last backup (if the file has an entry)
        // exclusive access to current snapshot to check last update date (read only)
        {
            let current_snapshot = &self.current_snapshot.lock().unwrap();
            if let Some(entry) =
                current_snapshot.get_entry_for_file_in_root(parent_root.name(), path_identifier)
                && fs_last_modified <= entry.last_modified()
            {
                debug!("The file {path_identifier} has not been modified, skipping");
                return Ok(());
            }
        }

        // Read file hash
        let hash_str = evaluate_file_sha512_hash(path).map_err(StoreError::ErrorDuringHashEval)?;
        if hash_str.is_empty() {
            panic!("Hash is empty! This cannot happen.");
        }

        // heavy lifting - exclusive access to data_store_cache for checking and updating contents
        {
            let mut data_store_cache = self.data_store_cache.lock().unwrap();
            if !data_store_cache.contains(&hash_str) {
                data_store_cache.insert(hash_str.clone());
                drop(data_store_cache);

                debug!("New hash found: {hash_str}\nCompressing and storing the file");
                // Compress and store the file
                self.cache_data_handler
                    .compress_and_store_file(path, &hash_str)
                    .map_err(StoreError::CacheHandlerError)?;
            }
        }

        // update snapshot entry
        {
            let mut current_snapshot = self.current_snapshot.lock().unwrap();
            let entries = current_snapshot.get_root_entries_mut(parent_root.name());
            let new_snapshot_entry = SnapshotEntry::new(hash_str, fs_last_modified);
            entries.insert(path_identifier.to_string(), new_snapshot_entry);
        }
        debug!("Updating snapshot record!");
        Ok(())
    }

    pub fn restore_file(&self, file_id: &str, parent_root: &RootDefinition) -> StoreResult<()> {
        let hash_str = {
            let current_snapshot = self.current_snapshot.lock().unwrap();
            current_snapshot
                .get_entry_for_file_in_root(parent_root.name(), file_id)
                .ok_or(StoreError::StoreEntryNotFound)?
                .data_hash()
                .to_string()
        };

        let destination = original_file_path_from_identifier(file_id, parent_root);

        self.cache_data_handler
            .uncompress_and_restore(&destination, &hash_str)
            .map_err(StoreError::CacheHandlerError)?;
        Ok(())
    }

    pub fn get_snapshot(&self) -> MutexGuard<'_, Snapshot> {
        self.current_snapshot.lock().unwrap()
    }

    pub fn swap_snapshot(&self, new_snapshot: &mut Snapshot) {
        let lock = &mut *self.current_snapshot.lock().unwrap();
        swap(lock, new_snapshot);
    }

    pub fn pop_snapshot(&self) -> Snapshot {
        let mut replacement = Snapshot::empty();
        self.swap_snapshot(&mut replacement);
        replacement
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
    use galvanizer_config::{
        Config, compression::CompressionOptions, root_definition::RootDefinition,
        store_preferences::StorePreferences,
    };

    use crate::{
        snapshots::snapshot::Snapshot,
        store::{Store, UninitializedStore},
        tests::resources,
    };

    fn mock_root() -> RootDefinition {
        let test_backup = resources().join("example_files");
        RootDefinition::new("example_root".into(), test_backup, None)
    }

    fn mock_config() -> Config {
        let test_dir = resources().join("store_example");
        Config::new(
            "1.2.3".into(),
            StorePreferences::new(test_dir, None),
            Some(CompressionOptions::Snap),
            None,
            vec![mock_root()],
        )
        .unwrap()
    }

    fn mock_store() -> UninitializedStore {
        let config = mock_config();
        Store::new_uninitialized_store(config, Snapshot::empty())
    }

    #[test]
    fn test_store() {
        init();
        let mut store = mock_store().initialize_hash_cache().unwrap();

        let file_path = resources()
            .join("example_files")
            .join("subfolder")
            .join("some_file.txt");
        let root = mock_root();
        store.store_file(&file_path, &root).unwrap();
        store.store_file(&file_path, &root).unwrap();
        store.store_file(&file_path, &root).unwrap();

        println!("{:?}", store.current_snapshot);
    }
}
