mod backup_restore;
mod fs_utils;

pub(crate) fn init_test_logger() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .try_init();
}
