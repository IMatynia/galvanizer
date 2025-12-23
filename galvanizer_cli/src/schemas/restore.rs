use clap::Args;
use galvanizer_store::snapshot_walker;

#[derive(Debug, Args)]
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

impl From<RestoreArgs> for snapshot_walker::RestoreArgs {
    fn from(val: RestoreArgs) -> Self {
        snapshot_walker::RestoreArgs {
            snapshot_id: val.snapshot_id,
            root_id: val.root_id,
            file_id: val.file_id,
            dont_overwrite_files: val.dont_overwrite_files,
            skip_missing_file_restore: val.skip_missing_file_restore,
            delete_new_files: val.delete_new_files,
        }
    }
}
