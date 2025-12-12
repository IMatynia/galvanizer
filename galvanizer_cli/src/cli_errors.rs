use galvanizer_config::config::ConfigError;
use galvanizer_store::{snapshots::snapshot::SnapshotError, store_error::StoreError};
use walkdir;

#[derive(Debug)]
pub enum CLIError {
    SnapshotError(SnapshotError),
    ConfigError(ConfigError),
    StoreError(StoreError),
    DirWalkError(walkdir::Error),
}

pub type CLIResult<T> = Result<T, CLIError>;
