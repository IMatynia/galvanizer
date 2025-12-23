use crate::snapshots::snapshot::{Snapshot, SnapshotError, SnapshotResult};
use chrono::{DateTime, NaiveDateTime, Utc};
use galvanizer_config::Config;
use log::debug;
use std::{
    ffi::OsStr,
    fs::{self, read_dir},
    path::{Path, PathBuf},
};

const SNAPSHOT_DATE_FMT: &str = "%Y-%m-%d_%H:%M:%S%.3f";

impl Snapshot {
    /// Find the path of the latest snapshot. Returns None if no such file was found! Errors may also occur along the way.
    pub fn find_latest_snapshot(config: &Config) -> Result<Option<PathBuf>, SnapshotError> {
        let snapshots_dir = config
            .get_snaphots_path()
            .map_err(SnapshotError::ConfigError)?;
        let mut latest_snapshot_loc: Option<(NaiveDateTime, PathBuf)> = None;
        for entry_result in fs::read_dir(snapshots_dir).map_err(SnapshotError::SnapshotIOError)? {
            let file = entry_result.map_err(SnapshotError::SnapshotIOError)?;
            let filename = file
                .path()
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .ok_or(SnapshotError::SnapshotEntryReadError)?;
            let datetime = chrono::NaiveDateTime::parse_from_str(&filename, SNAPSHOT_DATE_FMT)
                .map_err(SnapshotError::InvalidSnapshotName)?;

            if let Some((datetime_best, path_best)) = &mut latest_snapshot_loc {
                if datetime > *datetime_best {
                    *datetime_best = datetime;
                    *path_best = file.path().to_path_buf();
                }
            } else {
                latest_snapshot_loc = Some((datetime, file.path().to_path_buf()))
            }
        }

        Ok(latest_snapshot_loc.map(|(_, path)| path))
    }

    pub fn get_snapshot_path_from_id(
        config: &Config,
        id: String,
    ) -> Result<PathBuf, SnapshotError> {
        let snapshots_dir = config
            .get_snaphots_path()
            .map_err(SnapshotError::ConfigError)?;
        Ok(snapshots_dir.join(id).with_extension("toml"))
    }

    pub fn load_snapshot_from_path(path: &Path) -> Result<Snapshot, SnapshotError> {
        let bytes = fs::read(path).map_err(SnapshotError::SnapshotIOError)?;
        toml::from_slice(&bytes).map_err(SnapshotError::SnapshotDeserializationError)
    }

    pub fn load_latest_latest_snapshot(config: &Config) -> Result<Snapshot, SnapshotError> {
        if let Some(path) = Self::find_latest_snapshot(config)? {
            // Load latest
            Self::load_snapshot_from_path(&path)
        } else {
            // No snapshots were found. Apparently there arent any!
            Ok(Snapshot::empty())
        }
    }

    pub fn load_snapshot_by_id(config: &Config, id: String) -> Result<Snapshot, SnapshotError> {
        let path = Self::get_snapshot_path_from_id(config, id)?;
        Self::load_snapshot_from_path(&path)
    }

    pub fn save_snaphot(snapshot: &Snapshot, config: &Config) -> Result<(), SnapshotError> {
        let now: DateTime<Utc> = Utc::now();
        let snapshot_path =
            Self::get_snapshot_path_from_id(config, now.format(SNAPSHOT_DATE_FMT).to_string())?;
        fs::write(
            snapshot_path,
            toml::to_string_pretty(&snapshot).map_err(SnapshotError::SnapshotSerializationError)?,
        )
        .map_err(SnapshotError::SnapshotIOError)?;
        Ok(())
    }

    pub fn load_all_snapshot_ids(config: &Config) -> SnapshotResult<impl Iterator<Item = String>> {
        let snapshots_dir = config
            .get_snaphots_path()
            .map_err(SnapshotError::ConfigError)?;
        Ok(read_dir(snapshots_dir)
            .map_err(SnapshotError::SnapshotIOError)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().is_file() && entry.path().extension() == Some(OsStr::new("toml"))
            })
            .filter_map(|entry| entry.path().file_stem().map(|s| s.display().to_string())))
    }

    pub fn load_by_id_or_latest(
        config: &Config,
        id: Option<String>,
    ) -> Result<Snapshot, SnapshotError> {
        if let Some(snapshot_id) = id {
            debug!("Loading snapshot {snapshot_id}");
            Snapshot::load_snapshot_by_id(config, snapshot_id)
        } else {
            debug!("Loading latest snapshot");
            Snapshot::load_latest_latest_snapshot(config)
        }
    }
}
