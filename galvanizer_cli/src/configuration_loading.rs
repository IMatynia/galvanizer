use std::{env::home_dir, fs, io, path::PathBuf};

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

fn default_config_path() -> Option<PathBuf> {
    home_dir().map(|x| x.join(".galvanizer.toml"))
}

pub fn load_app_config(path: Option<PathBuf>) -> Result<Config, ConfigLoadingErrors> {
    let path = path
        .or(default_config_path())
        .ok_or(ConfigLoadingErrors::CouldNotAccessHomeDirectory)?;

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
