use dialoguer::{Input, Select};
use galvanizer_config::{
    Config, compression::CompressionOptions, config::CURRENT_VERISON,
    root_definition::RootDefinition, store_preferences::StorePreferences,
};
use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::configuration_loading::ConfigLoadingErrors;

pub fn first_time_config_customization_prompt(path: &Path) -> Result<(), ConfigLoadingErrors> {
    println!("No config detected at {path:?}. Anwser these questions to create new setup!");
    let store_root: String = Input::new()
        .with_prompt("Backup store path")
        .default("~/.galvanizer_backup".into())
        .validate_with(|input: &String| {
            PathBuf::from_str(input)
                .map(|_| ())
                .map_err(|e| format!("Invalid path: {e}"))
        })
        .interact_text()
        .expect("UI");
    // Unwrap, because we did validation earlier
    let store_root = PathBuf::from_str(&store_root).unwrap();

    println!(
        "Now add backup roots. Provide paths to directories you want to backup! Leave the field blank to skip this step or to finish selection."
    );
    let mut backup_roots: Vec<(String, PathBuf)> = vec![];
    loop {
        let root_id: String = Input::new()
            .with_prompt("New root name")
            .validate_with(|input: &String| {
                if input.chars().all(|c| c.is_alphanumeric()) {
                    Ok(())
                } else {
                    Err("Not alphanumeric!")
                }
            })
            .default("".into())
            .interact_text()
            .expect("UI");

        if root_id.is_empty() {
            break;
        }

        let backup_root: String = Input::new()
            .with_prompt("New root location")
            .validate_with(|input: &String| {
                if input.is_empty() {
                    return Ok(());
                }
                PathBuf::from_str(input)
                    .map(|_| ())
                    .map_err(|e| format!("Invalid path: {e}"))
            })
            .default("".into())
            .interact_text()
            .expect("UI");

        if backup_root.is_empty() {
            break;
        }

        // Validation was done earlier
        backup_roots.push((root_id, PathBuf::from_str(&backup_root).unwrap()));
    }

    println!("Choose compression algorithm");
    let compressions = ["Disabled", "Snap"];

    let selection = Select::new()
        .with_prompt("Compression")
        .default(1)
        .items(compressions)
        .interact()
        .expect("UI");

    let compression = match selection {
        0 => CompressionOptions::Disabled,
        1 => CompressionOptions::Snap,
        _ => CompressionOptions::Disabled,
    };

    let default_config = Config::new(
        CURRENT_VERISON.into(),
        StorePreferences::new(store_root, None),
        Some(compression),
        None,
        backup_roots
            .into_iter()
            .map(|(root_id, root_path)| RootDefinition::new(root_id, root_path, None))
            .collect(),
    )
    .map_err(ConfigLoadingErrors::DefaultConfigError)?;

    // Write the default config into the config location
    let content = toml::to_string_pretty(&default_config)
        .map_err(ConfigLoadingErrors::ConfigSerializationError)?;
    fs::write(path, content).map_err(ConfigLoadingErrors::ConfigWriteIOError)?;
    println!("The new config has been saved at {path:?}");
    Ok(())
}
