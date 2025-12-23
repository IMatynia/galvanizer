use std::io;

use galvanizer_config::config::ConfigError;

use crate::store_cache::file_io::CacheHandlerError;

#[derive(Debug)]
pub enum StoreError {
    InvalidOptionProvided { detail: String },
    ConfigurationError(ConfigError),
    CannotReadStoreCacheFolder(io::Error),
    PathIdentifierError(&'static str),
    ErrorDuringHashEval(io::Error),
    CriticalHashError,
    FileMetadataError(io::Error),
    CacheHandlerError(CacheHandlerError),
    StoreEntryNotFound,
    DataCacheHasNotBeenRefreshed,
}

pub type StoreResult<T> = Result<T, StoreError>;
