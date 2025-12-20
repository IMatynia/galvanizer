use std::{fs, io, path::PathBuf};

use galvanizer_config::{Config, config::ConfigError};

use crate::first_time_config_prompt::first_time_config_customization_prompt;

pub enum ConfigLoadingErrors {
    CouldNotAccessHomeDirectory,
    ConfigReadIOError(io::Error),
    ConfigDeseralizationError(toml::de::Error),
    DefaultConfigError(ConfigError),
    ConfigSerializationError(toml::ser::Error),
    ConfigWriteIOError(io::Error),
}

pub fn load_app_config(path: PathBuf) -> Result<Config, ConfigLoadingErrors> {
    if !path.exists() {
        first_time_config_customization_prompt(&path)?;
    }

    toml::from_slice(
        fs::read(path)
            .map_err(ConfigLoadingErrors::ConfigReadIOError)?
            .as_slice(),
    )
    .map_err(ConfigLoadingErrors::ConfigDeseralizationError)
}
