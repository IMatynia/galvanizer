use std::path::PathBuf;

pub(crate) fn resources() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("tests")
        .join("resources")
}
