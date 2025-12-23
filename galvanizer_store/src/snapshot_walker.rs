use std::{iter::once, path::PathBuf};

use either::Either;
use globset::Glob;

use crate::snapshots::snapshot::Snapshot;

pub struct SnapshotWalkerFilter {
    /// Snapshot date to restore from. List all snapshots to check the available dates! If no date is given, the most recent snapshot is chosen.
    pub snapshot_id: Option<String>,
    /// If defined, restores only this root, otherwise restores all roots
    pub root_id: Option<String>,
    /// If defined, restores only files that match this glob, otherwise restores all files
    pub file_id: Option<String>,
    /// Skip files that are on disk and in the backup. Doesnt overwrite the current version of a given file with the one from the backup.
    pub dont_overwrite_files: bool,
    /// Skip restoring files that are completly missing - for example, you deleted a file and dont want it to come back upon backup restore.
    pub skip_missing_file_restore: bool,
    /// This option makes it so that files that are on the disk but are not in the backup at all will be deleted, to restore the file structure faithfully.
    pub delete_new_files: bool,
}

impl SnapshotWalkerFilter {
    pub fn all() -> Self {
        Self {
            snapshot_id: None,
            root_id: None,
            file_id: None,
            dont_overwrite_files: false,
            skip_missing_file_restore: false,
            delete_new_files: false,
        }
    }
}

#[derive(Debug)]
pub enum SnapshotWalkerErr {
    RootNameDoesNotMatchCriteria { root_id: String, filter: String },
    FileNameDoesNotMatchGlob { root_id: String, file_id: String },
    InvalidRoot { root_id: String },
    InvalidFileGlob(globset::Error),
    StoreEntryNotFound(String),
}

pub struct SnapshotWalkerItem {
    pub root_id: String,
    pub path: PathBuf,
    pub hash_str: String,
}

pub fn walk_all_files_in_snapshot(snapshot: &Snapshot) -> impl Iterator<Item = SnapshotWalkerItem> {
    snapshot.get_roots().values().flat_map(|root_entry| {
        root_entry.entries().iter().map(|(file_id, entry)| {
            let hash_str = entry.data_hash().to_string();
            let destination = root_entry
                .root_definition()
                .make_original_file_path_from_identifier(file_id);
            SnapshotWalkerItem {
                root_id: root_entry.root_definition().name().to_string(),
                path: destination,
                hash_str,
            }
        })
    })
}

pub fn walk_all_files_in_snapshot_filtered(
    snapshot: &Snapshot,
    filter: &SnapshotWalkerFilter,
) -> impl Iterator<Item = Result<SnapshotWalkerItem, SnapshotWalkerErr>> {
    snapshot
        .get_roots()
        .values()
        .map(|entry| (entry.root_definition(), entry.entries()))
        .flat_map(|(root, entries)| {
            // Silent skip on root id mismatch
            if let Some(root_filter) = &filter.root_id
                && (root_filter != root.name())
            {
                return Either::Left(once(Err(SnapshotWalkerErr::RootNameDoesNotMatchCriteria {
                    root_id: root.name().to_string(),
                    filter: root_filter.to_owned(),
                })));
            }

            Either::Right(entries.iter().filter_map(|(file_id, entry)| {
                // Check for file glob
                if let Some(file_glob) = &filter.file_id {
                    match Glob::new(file_glob) {
                        Ok(glob) => {
                            // Check if the file matches the glob
                            if !glob.compile_matcher().is_match(file_id) {
                                // Silent skip
                                return None;
                            }
                        }
                        // Glob was invalid
                        Err(e) => return Some(Err(SnapshotWalkerErr::InvalidFileGlob(e))),
                    }
                }

                // Query for hash string
                let hash_str = entry.data_hash().to_string();
                let destination = root.make_original_file_path_from_identifier(file_id);

                let file_currently_exists = destination.exists();

                if file_currently_exists && filter.dont_overwrite_files {
                    return None;
                }

                if !file_currently_exists && filter.skip_missing_file_restore {
                    return None;
                }

                if filter.delete_new_files {
                    todo!("Option delete new files is not implemented!");
                }

                // All checks passed, send file to be restored
                Some(Ok(SnapshotWalkerItem {
                    root_id: root.name().to_string(),
                    path: destination,
                    hash_str,
                }))
            }))
        })
}
