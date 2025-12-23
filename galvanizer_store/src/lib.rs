#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::panic,))]
/// # Galvanizer data store
///
/// ## Store directory structure
///
/// ```text
/// <store_root>
/// |
/// |-> data_store
/// |   |-> <base64 hash prefix>
/// |   |   |-> <base64 hash of uncompressed file as filename> -> compressed content of a file identified by the hash
/// |   |   |...
/// |   |-> <base64 hash prefix>
/// |   |...
/// |-> snapshots
///     |-> <snapshot date>.toml -> file structure of the given root. Contains a list of relative paths, names and file hashes for lookup
///     |-> <another snapshot>.toml
///     | ...   
/// ```
pub mod root_walker;
pub mod snapshot_walker;
pub mod snapshots;
pub mod store_cache;
pub mod store_error;
#[cfg(test)]
mod tests;
