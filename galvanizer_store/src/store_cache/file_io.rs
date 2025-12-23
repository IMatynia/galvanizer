use std::{
    fs::{File, create_dir_all},
    io::{self, BufReader, BufWriter},
    path::Path,
};

use galvanizer_config::{Config, config::ConfigError, root_definition::RootDefinition};

use crate::{
    snapshots::{shapshot_delta::SnapshotDelta, shapshot_entry::SnapshotEntry},
    store_cache::{
        file_property_utils::{evaluate_file_sha512_hash, get_file_last_modified_date},
        hash_db::HashDB,
    },
    store_error::{StoreError, StoreResult},
};

#[derive(Debug)]
pub enum CacheHandlerError {
    FailedToGetCacheLocation(ConfigError),
    FailedToOpenFile(io::Error, &'static str),
    ErrorDuringCompression(io::Error),
}

pub type CacheHandlerResult<T> = Result<T, CacheHandlerError>;

fn compress_and_store_file(
    config: &Config,
    path: &Path,
    hash_str: &str,
) -> CacheHandlerResult<u64> {
    let destination_path = config
        .get_store_cache_path_for_hash(hash_str)
        .map_err(CacheHandlerError::FailedToGetCacheLocation)?;

    let mut input = BufReader::new(
        File::open(path).map_err(|e| CacheHandlerError::FailedToOpenFile(e, "file to compress"))?,
    );
    let output = BufWriter::new(
        File::create(destination_path)
            .map_err(|e| CacheHandlerError::FailedToOpenFile(e, "compressed file destination"))?,
    );
    // Wrap the stdout writer in a Snappy writer.
    let mut output_compressed = snap::write::FrameEncoder::new(output);
    io::copy(&mut input, &mut output_compressed).map_err(CacheHandlerError::ErrorDuringCompression)
}

pub fn store_file(
    config: &Config,
    data_store_cache: &HashDB,
    path: &Path,
    parent_root: &RootDefinition,
) -> StoreResult<SnapshotDelta> {
    // Query for last modified date
    let fs_last_modified =
        get_file_last_modified_date(path).map_err(StoreError::FileMetadataError)?;

    // Get path identifier str
    let path_identifier = parent_root
        .make_path_identifier(path)
        .map_err(StoreError::PathIdentifierError)?;

    // Read file hash
    let hash_str = evaluate_file_sha512_hash(path).map_err(StoreError::ErrorDuringHashEval)?;
    if hash_str.is_empty() {
        return Err(StoreError::CriticalHashError);
    }

    // heavy lifting - exclusive access to data_store_cache for checking and updating contents
    if data_store_cache.insert(hash_str.clone()) {
        // Compress and store the file
        compress_and_store_file(config, path, &hash_str).map_err(StoreError::CacheHandlerError)?;
    }

    Ok(SnapshotDelta {
        root: parent_root.clone(),
        file_id: path_identifier.to_string(),
        entry: SnapshotEntry::new(hash_str, fs_last_modified),
    })
}
pub fn restore_file(
    config: &Config,
    destination_path: &Path,
    hash_str: &str,
) -> CacheHandlerResult<u64> {
    let source_file = config
        .get_store_cache_path_for_hash(hash_str)
        .map_err(CacheHandlerError::FailedToGetCacheLocation)?;

    if let Some(parent) = destination_path.parent() {
        create_dir_all(parent).map_err(|e| {
            CacheHandlerError::FailedToOpenFile(
                e,
                "error while trying to create the destination folder",
            )
        })?;
    }

    let input =
        BufReader::new(File::open(source_file).map_err(|e| {
            CacheHandlerError::FailedToOpenFile(e, "compressed file to uncompress")
        })?);
    let mut output = BufWriter::new(
        File::create(destination_path)
            .map_err(|e| CacheHandlerError::FailedToOpenFile(e, "restored file destination"))?,
    );
    // Wrap the stdout writer in a Snappy writer.
    let mut input_uncompressed = snap::read::FrameDecoder::new(input);
    io::copy(&mut input_uncompressed, &mut output)
        .map_err(CacheHandlerError::ErrorDuringCompression)
}
