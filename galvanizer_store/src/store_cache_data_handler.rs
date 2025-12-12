use std::{
    fs::File,
    io::{self, BufReader, BufWriter},
    path::Path,
    sync::Arc,
};

use galvanizer_config::{Config, config::ConfigError};

pub struct StoreCacheDataHandler {
    config: Arc<Config>,
}

#[derive(Debug)]
pub enum CacheHandlerError {
    FailedToGetCacheLocation(ConfigError),
    FailedToOpenFile(io::Error),
    ErrorDuringCompression(io::Error),
}

pub type CacheHandlerResult<T> = Result<T, CacheHandlerError>;

impl StoreCacheDataHandler {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    pub fn compress_and_store_file(&self, path: &Path, hash_str: &str) -> CacheHandlerResult<u64> {
        let destination_path = self
            .config
            .get_store_cache_path_for_hash(hash_str)
            .map_err(CacheHandlerError::FailedToGetCacheLocation)?;

        let mut input =
            BufReader::new(File::open(path).map_err(CacheHandlerError::FailedToOpenFile)?);
        let output = BufWriter::new(
            File::open(destination_path).map_err(CacheHandlerError::FailedToOpenFile)?,
        );
        // Wrap the stdout writer in a Snappy writer.
        let mut output_compressed = snap::write::FrameEncoder::new(output);
        io::copy(&mut input, &mut output_compressed)
            .map_err(CacheHandlerError::ErrorDuringCompression)
    }

    pub fn uncompress_and_restore(
        &self,
        destination_path: &Path,
        hash_str: &str,
    ) -> CacheHandlerResult<u64> {
        let source_file = self
            .config
            .get_store_cache_path_for_hash(hash_str)
            .map_err(CacheHandlerError::FailedToGetCacheLocation)?;

        let input =
            BufReader::new(File::open(source_file).map_err(CacheHandlerError::FailedToOpenFile)?);
        let mut output = BufWriter::new(
            File::open(destination_path).map_err(CacheHandlerError::FailedToOpenFile)?,
        );
        // Wrap the stdout writer in a Snappy writer.
        let mut input_uncompressed = snap::read::FrameDecoder::new(input);
        io::copy(&mut input_uncompressed, &mut output)
            .map_err(CacheHandlerError::ErrorDuringCompression)
    }
}
