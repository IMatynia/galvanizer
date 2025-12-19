use std::{
    fs::{self, create_dir_all, remove_dir_all},
    path::{Path, PathBuf},
};

#[derive(Clone)]
pub struct TestFile {
    pub path: &'static str,
    pub content: &'static str,
}

impl TestFile {
    pub fn real_path(&self, root: &Path) -> PathBuf {
        root.to_path_buf().join(self.path)
    }
}

pub fn build_file_structure(root: &Path, files: &[TestFile]) {
    for f in files {
        let new_path = f.real_path(root);
        create_dir_all(new_path.parent().unwrap()).unwrap();
        fs::write(new_path, f.content).unwrap();
    }
}

pub fn assert_files_are_correct(root: &Path, files: &[TestFile]) {
    files.iter().for_each(|f| {
        let real_path = f.real_path(root);
        assert!(real_path.exists());
        assert_eq!(fs::read_to_string(real_path).unwrap(), f.content);
    })
}

pub fn clean_ws(root: &Path) {
    remove_dir_all(root).unwrap();
}

pub mod examples {
    use crate::tests::fs_utils::TestFile;

    pub fn empty() -> &'static [TestFile] {
        &[]
    }

    pub fn all_examples_at_once() -> impl Iterator<Item = TestFile> {
        [].iter()
            .chain(one_empty_file())
            .chain(one_small_file())
            .chain(two_repeating_files())
            .chain(simple_subdirectory())
            .chain(simple_subdirectory())
            .chain(complex_dir_structure())
            .chain(different_extensions())
            .map(|x| x.clone())
    }

    pub fn one_empty_file() -> &'static [TestFile] {
        &[TestFile {
            path: "file.txt",
            content: "",
        }]
    }

    pub fn one_small_file() -> &'static [TestFile] {
        &[TestFile {
            path: "file_small.txt",
            content: "Hello world! ąćźńżół",
        }]
    }

    pub fn two_repeating_files() -> &'static [TestFile] {
        &[
            TestFile {
                path: "file_a.txt",
                content: "repeating content",
            },
            TestFile {
                path: "file_b.txt",
                content: "repeating content",
            },
        ]
    }

    pub fn simple_subdirectory() -> &'static [TestFile] {
        &[TestFile {
            path: "example_dir/sub_file.txt",
            content: "subfile content",
        }]
    }

    pub fn complex_dir_structure() -> &'static [TestFile] {
        &[
            TestFile {
                path: ".hidden_file",
                content: "secrets",
            },
            TestFile {
                path: ".hidden/other.txt",
                content: "secrets again",
            },
            TestFile {
                path: "complex_folder/subfolder/subsub/subsubsub/file.txt",
                content: "deep",
            },
            TestFile {
                path: "complex_folder/file.txt",
                content: "shallow",
            },
            TestFile {
                path: "another_folder/with_a_subfolder/file.txt",
                content: "another one",
            },
        ]
    }

    pub fn different_extensions() -> &'static [TestFile] {
        &[
            TestFile {
                path: "file.a",
                content: "a",
            },
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
        ]
    }
}
