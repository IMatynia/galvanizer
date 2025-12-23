use std::{iter::once, path::PathBuf};

use either::Either;
use galvanizer_config::Config;
use globset::Glob;

use crate::snapshots::snapshot::Snapshot;

pub struct RestoreArgs {
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

#[derive(Debug)]
pub enum SnapshotWalkerErr {
    RootNameDoesNotMatchCriteria { root_id: String, filter: String },
    FileNameDoesNotMatchGlob { root_id: String, file_id: String },
    InvalidRoot { root_id: String },
    InvalidFileGlob(globset::Error),
    StoreEntryNotFound(String),
}

pub enum SnapshotWalkerJob {
    RestoreFile {
        destination: PathBuf,
        hash_str: String,
    },
}

pub fn walk_all_files_in_snapshot<'a>(
    config: &'a Config,
    snapshot: &'a Snapshot,
    restoration_options: &'a RestoreArgs,
) -> impl Iterator<Item = Result<SnapshotWalkerJob, SnapshotWalkerErr>> {
    snapshot
        .get_root_ids()
        .filter_map(|root_id| config.find_root_by_id(root_id))
        .flat_map(|parent_root| {
            // Silent skip on root id mismatch
            if let Some(root_filter) = &restoration_options.root_id
                && (root_filter != parent_root.name())
            {
                return Either::Left(once(Err(SnapshotWalkerErr::RootNameDoesNotMatchCriteria {
                    root_id: parent_root.name().to_string(),
                    filter: root_filter.to_owned(),
                })));
            }

            if let Some(files) = snapshot.get_rel_paths_for_root(parent_root.name()) {
                Either::Right(files.filter_map(|file_id| {
                    // Check for file glob
                    if let Some(file_glob) = &restoration_options.file_id {
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
                    let hash_str = {
                        if let Some(entry) =
                            snapshot.get_entry_for_file_in_root(parent_root.name(), file_id)
                        {
                            entry.data_hash().to_string()
                        } else {
                            return Some(Err(SnapshotWalkerErr::StoreEntryNotFound(
                                file_id.clone(),
                            )));
                        }
                    };

                    let destination = parent_root.make_original_file_path_from_identifier(file_id);
                    let file_currently_exists = destination.exists();

                    if file_currently_exists && restoration_options.dont_overwrite_files {
                        return None;
                    }

                    if !file_currently_exists && restoration_options.skip_missing_file_restore {
                        return None;
                    }

                    if restoration_options.delete_new_files {
                        todo!("Option delete new files is not implemented!");
                    }

                    // All checks passed, send file to be restored
                    Some(Ok(SnapshotWalkerJob::RestoreFile {
                        destination,
                        hash_str,
                    }))
                }))
            } else {
                Either::Left(once(Err(SnapshotWalkerErr::InvalidRoot {
                    root_id: parent_root.name().into(),
                })))
            }
        })
}
