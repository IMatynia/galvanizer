#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        VERSION,
        cli::restore::RestoreArgs,
        commands::{
            backup::{self, run},
            restore,
        },
        tests::{
            fs_utils::{
                TestFile, assert_files_are_correct, build_file_structure, clean_ws,
                examples::{all_examples_at_once, complex_dir_structure, one_small_file},
            },
            init_test_logger, resources,
        },
    };
    use galvanizer_config::{
        Config, root_definition::RootDefinition, store_preferences::StorePreferences,
    };
    use tempfile::tempdir;

    fn make_config_basic(data_root: PathBuf, store_root: PathBuf) -> Config {
        Config::new(
            VERSION.into(),
            StorePreferences::new(store_root, None),
            None,
            None,
            vec![RootDefinition::new("example_root".into(), data_root, None)],
        )
        .unwrap()
    }

    #[test]
    fn test_backup_and_restore_simple() {
        init_test_logger();
        let files: Vec<TestFile> = all_examples_at_once().collect();

        let test_temp_dir = tempdir().unwrap();
        let data_root = test_temp_dir.path().to_path_buf().join("data");
        let store_root = test_temp_dir.path().to_path_buf().join("store");

        build_file_structure(&data_root, &files);
        // Sanity check
        assert_files_are_correct(&data_root, &files);

        let config = make_config_basic(data_root.clone(), store_root);
        backup::run(config.clone()).unwrap();

        clean_ws(&data_root);

        restore::run(
            config.clone(),
            RestoreArgs {
                snapshot_id: None,
                root_id: None,
                file_id: None,
                dont_overwrite_files: false,
                skip_missing_file_restore: false,
                delete_new_files: false,
            },
        )
        .unwrap();

        assert_files_are_correct(&data_root, &files);
    }
}
