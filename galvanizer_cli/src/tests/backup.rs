#[cfg(test)]
mod tests {
    use crate::{commands::backup::run, tests::resources};
    use galvanizer_config::{
        Config, root_definition::RootDefinition, store_preferences::StorePreferences,
    };

    #[test]
    fn test_simple_backup() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .try_init();

        let resources_path = resources();

        let cfg = Config::new(
            "1.0".into(),
            StorePreferences::new(resources_path.clone().join("example_store"), None),
            None,
            None,
            vec![RootDefinition::new(
                "example".into(),
                resources_path.clone().join("example_files"),
                None,
            )],
        )
        .unwrap();

        run(cfg).unwrap();
    }
}
