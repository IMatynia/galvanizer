#[cfg(test)]
mod tests {
    use std::{fs::create_dir_all, path::PathBuf};

    use crate::{
        VERSION,
        commands::{backup::run::backup_command, restore::run::restore_command},
        tests::{
            fs_utils::{
                TestFile, assert_files_are_correct, assert_files_are_missing, build_file_structure,
                clean_ws,
                examples::{all_examples_at_once, different_extensions, empty},
            },
            init_test_logger,
        },
    };
    use galvanizer_config::{
        Config, root_definition::RootDefinition, store_preferences::StorePreferences,
    };
    use galvanizer_store::snapshot_walker::RestoreArgs;
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
        backup_command(config.clone()).unwrap();

        clean_ws(&data_root);

        restore_command(
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

    #[test]
    fn test_backup_and_restore_empty() {
        init_test_logger();
        let files: Vec<TestFile> = empty().into();

        let test_temp_dir = tempdir().unwrap();
        let data_root = test_temp_dir.path().to_path_buf().join("data");
        let store_root = test_temp_dir.path().to_path_buf().join("store");

        create_dir_all(&data_root).unwrap();

        build_file_structure(&data_root, &files);
        // Sanity check
        assert_files_are_correct(&data_root, &files);

        let config = make_config_basic(data_root.clone(), store_root);
        backup_command(config.clone()).unwrap();

        clean_ws(&data_root);

        restore_command(
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

    #[test]
    fn test_globbed_recovery() {
        init_test_logger();
        let files: Vec<TestFile> = different_extensions().into();

        let test_temp_dir = tempdir().unwrap();
        let data_root = test_temp_dir.path().to_path_buf().join("data");
        let store_root = test_temp_dir.path().to_path_buf().join("store");

        create_dir_all(&data_root).unwrap();

        build_file_structure(&data_root, &files);
        // Sanity check
        assert_files_are_correct(&data_root, &files);

        let config = make_config_basic(data_root.clone(), store_root);
        backup_command(config.clone()).unwrap();

        clean_ws(&data_root);

        restore_command(
            config.clone(),
            RestoreArgs {
                snapshot_id: None,
                root_id: None,
                file_id: Some("*.a".into()),
                dont_overwrite_files: false,
                skip_missing_file_restore: false,
                delete_new_files: false,
            },
        )
        .unwrap();

        assert_files_are_correct(
            &data_root,
            &[TestFile {
                path: "file.a",
                content: "a",
            }],
        );
        assert_files_are_missing(
            &data_root,
            &[
                TestFile {
                    path: "file.b",
                    content: "b",
                },
                TestFile {
                    path: "file.c",
                    content: "c",
                },
                TestFile {
                    path: "file.d",
                    content: "d",
                },
            ],
        );
    }
}
