#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::panic,
    )
)]

pub mod snapshots;
pub mod store;
pub mod store_cache_data_handler;
pub mod store_cache_tools;
pub mod store_error;
#[cfg(test)]
mod tests;
