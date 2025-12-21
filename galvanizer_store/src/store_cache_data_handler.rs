use std::{
    fs::{File, create_dir_all},
    io::{self, BufReader, BufWriter},
    path::Path,
};

use galvanizer_config::{Config, config::ConfigError};

#[derive(Debug)]
pub enum CacheHandlerError {
    FailedToGetCacheLocation(ConfigError),
    FailedToOpenFile(io::Error, &'static str),
    ErrorDuringCompression(io::Error),
}

pub type CacheHandlerResult<T> = Result<T, CacheHandlerError>;

pub fn compress_and_store_file(
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

pub fn uncompress_and_restore(
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
