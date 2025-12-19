#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::panic,
    )
)]

pub mod compression;
pub mod config;
pub mod root_definition;
pub mod root_preferences;
pub mod store_preferences;

#[cfg(test)]
mod tests;

pub use config::Config;
