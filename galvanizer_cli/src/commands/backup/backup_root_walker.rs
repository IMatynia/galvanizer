use std::path::PathBuf;
use galvanizer_config::Config;
use galvanizer_store::{
    snapshots::{shapshot_delta::SnapshotDelta, snapshot::Snapshot},
    store_cache_tools::{get_file_last_modified_date, get_path_identifier},
};
use rayon::iter::Either;
use walkdir::WalkDir;
use crate::commands::backup::{backup_command_schemas::BackupJob, walker_thread::WalkerError};

pub fn root_walker<'a>(
    config: &'a Config,
    old_snapshot: &'a Snapshot,
) -> impl Iterator<Item = Result<BackupJob, WalkerError>> + 'a {
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

                    let path_identifier = match get_path_identifier(&path, &root) {
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
                            root_id: root.name().to_string(),
                            path_identifier: path_identifier.to_string(),
                            entry,
                        };
                        return Some(Ok(BackupJob::AddCoppiedEntry(coppied_entry)));
                    }

                    Some(Ok(BackupJob::ProcessFileFurther {
                        path,
                        root: root.clone(),
                    }))
                }),
        )
    })
}
