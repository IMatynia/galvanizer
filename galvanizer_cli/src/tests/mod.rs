mod fs_utils;
use std::path::PathBuf;

mod backup_restore;

pub(crate) fn resources() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("resources")
}

pub(crate) fn init_test_logger() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .try_init();
}
