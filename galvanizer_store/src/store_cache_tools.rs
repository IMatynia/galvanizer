use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use chrono::{DateTime, Utc};
use galvanizer_config::root_definition::RootDefinition;
use sha2::{Digest, Sha512};
use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

// Returns a base64 encoded sha512 of the provided file
pub fn evaluate_file_sha512_hash(path: &Path) -> Result<String, io::Error> {
    let mut file = File::open(path)?;

    let mut hasher = Sha512::new();
    let mut buffer = [0u8; 1024 * 8];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    Ok(URL_SAFE.encode(hasher.finalize().as_slice()))
}

pub fn get_file_last_modified_date(path: &Path) -> Result<DateTime<Utc>, io::Error> {
    let fs_last_modified: DateTime<Utc> = path.metadata()?.modified()?.into();
    Ok(fs_last_modified)
}

pub fn get_path_identifier<'a>(
    path: &'a Path,
    parent_root: &RootDefinition,
) -> Result<&'a str, &'static str> {
    let path_short = path
        .strip_prefix(parent_root.path())
        .map_err(|_| "File is not within the current root directory!")?;
    let path_identifier = path_short
        .as_os_str()
        .to_str()
        .ok_or("Failed to read path as utf-8 string")?;
    Ok(path_identifier)
}

pub fn original_file_path_from_identifier(file_id: &str, parent_root: &RootDefinition) -> PathBuf {
    parent_root.path().clone().join(file_id)
}

#[cfg(test)]
mod tests {
    use crate::{store_cache_tools::evaluate_file_sha512_hash, tests::resources};

    #[test]
    fn test_empty_file_hash() {
        let path = resources().join("example_files").join("empty.txt");
        let hash = evaluate_file_sha512_hash(&path).unwrap();
        assert_eq!(
            hash,
            "z4PhNX7vuL3xVChQ1m2AB9Yg5AULVxXcg_SpIdNs6c5H0NE8XYXysP-DGNKHfuwvY7kxvUdBeoGlODJ6-SfaPg=="
        );
    }

    #[test]
    fn check_if_hashes_match() {
        let path_a = resources().join("example_files").join("file_a.txt");
        let path_b = resources().join("example_files").join("file_a_copy.txt");

        assert_eq!(
            evaluate_file_sha512_hash(&path_a).unwrap(),
            evaluate_file_sha512_hash(&path_b).unwrap()
        );
    }

    #[test]
    fn hash_big_file() {
        let path = resources().join("example_files").join("20kb_file.txt");
        assert_eq!(
            evaluate_file_sha512_hash(&path).unwrap(),
            "_A_grDZmXiRSsRdwVC7zr3cxkSZ_P57HaoCLcPIB2Lq80kIc5tQkRciM9-CPFV9SC3gnDwbKkl-qKycQp11XuQ=="
        );
    }
}
