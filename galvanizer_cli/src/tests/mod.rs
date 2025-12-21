mod fs_utils;
mod backup_restore;

pub(crate) fn init_test_logger() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .try_init();
}
