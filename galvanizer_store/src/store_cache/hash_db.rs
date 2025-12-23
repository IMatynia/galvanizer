use std::{
    collections::HashSet,
    fs::{self, DirEntry},
    path::Path,
};

use dashmap::DashSet;

use crate::store_error::{StoreError, StoreResult};

pub struct HashDB {
    inner: DashSet<String>,
}

impl HashDB {
    pub fn new(inner: DashSet<String>) -> Self {
        Self { inner }
    }

    pub fn empty() -> Self {
        Self {
            inner: DashSet::new(),
        }
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

    pub fn update_hash_set_with_cache_contents(self, store_root: &Path) -> StoreResult<Self> {
        Self::store_cache_file_iterator(store_root)?
            .flat_map(|x| x.path().file_stem().map(|s| s.to_owned()))
            .map(|x| x.display().to_string())
            .fold(&self.inner, |set, hash| {
                set.insert(hash);
                set
            });
        Ok(self)
    }

    pub fn insert(&self, hash: String) -> bool {
        self.inner.insert(hash)
    }

    pub fn into_hashset(self) -> HashSet<String> {
        self.inner.into_iter().collect()
    }
}
