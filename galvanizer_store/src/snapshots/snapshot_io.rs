use crate::snapshots::snapshot::{Snapshot, SnapshotError};
use chrono::{DateTime, NaiveDateTime, Utc};
use galvanizer_config::Config;
use std::{fs, path::PathBuf};

const SNAPSHOT_DATE_FMT: &str = "%Y-%m-%d_%H:%M:%S%.3f";

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

pub fn load_latest_latest_snapshot(config: &Config) -> Result<Snapshot, SnapshotError> {
    if let Some(path) = find_latest_snapshot(config)? {
        // Load latest
        let bytes = fs::read(path).map_err(SnapshotError::SnapshotIOError)?;
        Ok(toml::from_slice(&bytes).map_err(SnapshotError::SnapshotDeserializationError)?)
    } else {
        // No snapshots were found. Apparently there arent any!
        Ok(Snapshot::empty())
    }
}

pub fn save_snaphot(snapshot: Snapshot, config: &Config) -> Result<(), SnapshotError> {
    let now: DateTime<Utc> = Utc::now();
    let snapshot_path = config
        .get_snaphots_path()
        .map_err(SnapshotError::ConfigError)?
        .join(now.format(SNAPSHOT_DATE_FMT).to_string())
        .with_extension("toml");
    fs::write(
        snapshot_path,
        toml::to_string_pretty(&snapshot).map_err(SnapshotError::SnapshotSerializationError)?,
    )
    .map_err(SnapshotError::SnapshotIOError)?;
    Ok(())
}
