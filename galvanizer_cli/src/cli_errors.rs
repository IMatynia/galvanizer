use std::path::PathBuf;

use crossbeam::channel::SendError;
use galvanizer_config::{config::ConfigError, root_definition::RootDefinition};
use galvanizer_store::{snapshots::snapshot::SnapshotError, store_error::StoreError};
use walkdir;

#[derive(Debug)]
pub enum CLIError {
    SnapshotError(SnapshotError),
    ConfigError(ConfigError),
    StoreError(StoreError),
    DirWalkError(walkdir::Error),
    ChannelDisconnected(SendError<(PathBuf, RootDefinition)>),

    TheadJoinError,
}

pub type CLIResult<T> = Result<T, CLIError>;
