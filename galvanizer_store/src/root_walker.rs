use crate::{
    snapshots::{shapshot_delta::SnapshotDelta, snapshot::Snapshot},
    store_cache::file_property_utils::get_file_last_modified_date,
};
use either::Either;
use galvanizer_config::{Config, config::ConfigError, root_definition::RootDefinition};
use std::{io, path::PathBuf};
use walkdir::WalkDir;

#[derive(Debug)]
pub enum WalkerError {
    WalkDirError(walkdir::Error),
    RootConfigurationError(ConfigError),
    RootDoesNotExist(String),
    PathIdentifierError(&'static str),
    ModificationDateReadError(io::Error),
}

pub enum WalkResult {
    UnchangedFile(SnapshotDelta),
    NewFile { path: PathBuf, root: RootDefinition },
}

pub fn walk_all_files_in_backup_roots<'a>(
    config: &'a Config,
    old_snapshot: &'a Snapshot,
) -> impl Iterator<Item = Result<WalkResult, WalkerError>> + 'a {
    config.backup_roots().iter().flat_map(move |root| {
        let root = root.clone();

        let root_prefs = match config.get_combined_preferences_for_root(root.name()) {
            Some(p) => p,
            None => {
                return Either::Left(std::iter::once(Err(WalkerError::RootDoesNotExist(
                    root.name().to_string(),
                ))));
            }
        };

        let include_set = match root_prefs.inclusions_globset() {
            Ok(s) => s,
            Err(e) => {
                return Either::Left(std::iter::once(Err(WalkerError::RootConfigurationError(e))));
            }
        };

        let exclude_set = match root_prefs.exclusions_globset() {
            Ok(s) => s,
            Err(e) => {
                return Either::Left(std::iter::once(Err(WalkerError::RootConfigurationError(e))));
            }
        };

        Either::Right(
            WalkDir::new(root.path())
                .into_iter()
                .filter_map(move |dir_entry| {
                    let dir_entry = match dir_entry {
                        Ok(e) => e,
                        Err(e) => return Some(Err(WalkerError::WalkDirError(e))),
                    };
                    // Silent skip - not a file
                    if !dir_entry.file_type().is_file() {
                        return None;
                    }
                    let path: PathBuf = dir_entry.into_path();
                    // Silent skip - no glob match
                    if !include_set.is_match(&path) {
                        return None;
                    }
                    // Silent skip - exclusion
                    if exclude_set.is_match(&path) {
                        return None;
                    }

                    let path_identifier = match root.make_path_identifier(&path) {
                        Ok(id) => id,
                        Err(e) => return Some(Err(WalkerError::PathIdentifierError(e))),
                    };

                    let fs_last_modified = match get_file_last_modified_date(&path) {
                        Ok(t) => t,
                        Err(e) => return Some(Err(WalkerError::ModificationDateReadError(e))),
                    };

                    let old_entry = old_snapshot
                        .get_entry_for_file_in_root(root.name(), path_identifier)
                        .cloned();
                    if let Some(entry) = old_entry
                        && fs_last_modified <= entry.last_modified()
                    {
                        let coppied_entry = SnapshotDelta {
                            root: root.clone(),
                            file_id: path_identifier.to_string(),
                            entry,
                        };
                        return Some(Ok(WalkResult::UnchangedFile(coppied_entry)));
                    }

                    Some(Ok(WalkResult::NewFile {
                        path,
                        root: root.clone(),
                    }))
                }),
        )
    })
}
