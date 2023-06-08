use std::path::PathBuf;

use cargo_metadata::camino::Utf8PathBuf;
use walkdir::WalkDir;

use bsp_types::requests::{SourceItem, SourceItemKind, SourcesItem};
use bsp_types::BuildTargetIdentifier;

use crate::project_model::target_details::{CargoTargetKind, TargetDetails};
use crate::utils::uri::file_uri;

pub fn get_sources_for_target(
    id: &BuildTargetIdentifier,
    target_details: TargetDetails,
) -> SourcesItem {
    let package_path = target_details.package_abs_path.clone();

    SourcesItem {
        target: id.clone(),
        sources: list_target_sources(target_details),
        roots: vec![file_uri(package_path)],
    }
}

fn list_target_sources(target_details: TargetDetails) -> Vec<SourceItem> {
    fn list_source_files_in_path(path: Utf8PathBuf) -> Vec<SourceItem> {
        get_all_rs_files_in_dir(path.as_str())
            .into_iter()
            .map(create_source_item)
            .collect()
    }

    let package_path = target_details.package_abs_path;

    let mut src_sources: Vec<SourceItem> = list_source_files_in_path(package_path.join("src"));

    match target_details.kind {
        CargoTargetKind::Lib | CargoTargetKind::Bin => {}
        CargoTargetKind::Test | CargoTargetKind::Example | CargoTargetKind::Bench => {
            src_sources.append(&mut list_source_files_in_path(package_path.join("tests")))
        }
    }

    src_sources
}

fn get_all_rs_files_in_dir(dir: &str) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|entry| {
            entry.ok().and_then(|e| match e.path().extension() {
                Some(ext) if ext == "rs" => Some(e.into_path()),
                _ => None,
            })
        })
        .collect()
}

fn create_source_item(source_path: PathBuf) -> SourceItem {
    let source_kind = if source_path.is_dir() {
        SourceItemKind::Directory
    } else {
        SourceItemKind::File
    };

    return SourceItem {
        uri: file_uri(source_path.to_str().unwrap()),
        kind: source_kind,
        generated: false,
    };
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::fs::File;
    use std::path::{Path, PathBuf};

    use cargo_metadata::camino::Utf8PathBuf;
    use insta::assert_json_snapshot;
    use tempfile::tempdir;

    use bsp_types::BuildTargetIdentifier;

    use crate::project_model::sources::get_sources_for_target;
    use crate::project_model::target_details::{CargoTargetKind, TargetDetails};

    const RUST_FILE_NAMES: [&str; 3] = ["test1.rs", "test2.rs", "test3.rs"];
    const NOT_RUST_FILE_NAMES: [&str; 3] = ["test1.txt", "test4", "test5.rs.java"];

    fn create_files(files_names: &[&str], dir: &Path) -> HashSet<PathBuf> {
        let files_paths = files_names
            .iter()
            .map(|name| dir.join(name))
            .collect::<Vec<PathBuf>>();
        let _ = files_paths
            .iter()
            .map(|path| File::create(path.clone()).unwrap())
            .collect::<Vec<File>>();

        files_paths.into_iter().collect()
    }

    fn create_files_in_dirs(files_names: &[&str], dirs: Vec<&PathBuf>) -> Vec<HashSet<PathBuf>> {
        dirs.iter()
            .map(|dir| create_files(files_names, dir.as_path()))
            .collect()
    }

    mod create_source_item {
        use bsp_types::requests::{SourceItem, SourceItemKind};

        use crate::project_model::sources::create_source_item;
        use crate::utils::uri::file_uri;

        use super::*;

        struct TestCase {
            path: PathBuf,
            expected: SourceItem,
        }
        #[test]
        fn test() {
            let temp_dir = tempdir().unwrap();
            let temp_dir_path = temp_dir.path().to_path_buf();
            let temp_file_path = temp_dir.path().join("test.rs");
            let _temp_file = File::create(temp_file_path.clone()).unwrap();

            let cases = vec![
                TestCase {
                    path: temp_dir_path.clone(),
                    expected: SourceItem {
                        uri: file_uri(temp_dir_path.to_str().unwrap()),
                        kind: SourceItemKind::Directory,
                        generated: false,
                    },
                },
                TestCase {
                    path: temp_file_path.clone(),
                    expected: SourceItem {
                        uri: file_uri(temp_file_path.to_str().unwrap()),
                        kind: SourceItemKind::File,
                        generated: false,
                    },
                },
            ];

            for case in cases {
                let source_item = create_source_item(case.path);
                assert_eq!(source_item, case.expected);
                assert!(source_item.uri.starts_with("file://"));
            }
        }
    }

    mod get_all_rs_files_in_dir_test {
        use std::collections::HashSet;
        use std::fs;

        use crate::project_model::sources::get_all_rs_files_in_dir;

        use super::*;

        struct TestCase<'a> {
            dir_path: &'a PathBuf,
            expected: HashSet<PathBuf>,
        }

        fn make_dir(dir_path: &Path, name: &str) -> PathBuf {
            let new_dir_path = dir_path.join(name);
            fs::create_dir(new_dir_path.clone()).unwrap();
            new_dir_path
        }

        #[test]
        fn test() {
            let dir_root = tempdir().unwrap();
            let dir_root_path = dir_root.into_path();
            let dir_root_a = make_dir(&dir_root_path, "a");
            let dir_root_a_b = make_dir(&dir_root_a, "b");
            let dir_root_b = make_dir(&dir_root_path, "b");
            let dir_root_b_b = make_dir(&dir_root_b, "b");

            let rust_files_paths = create_files_in_dirs(
                &RUST_FILE_NAMES,
                vec![&dir_root_path, &dir_root_a, &dir_root_a_b, &dir_root_b],
            );
            let _ = create_files_in_dirs(&NOT_RUST_FILE_NAMES, vec![&dir_root_b, &dir_root_b_b]);

            let test_cases = vec![
                TestCase {
                    dir_path: &dir_root_path,
                    expected: vec![
                        rust_files_paths[0].clone(),
                        rust_files_paths[1].clone(),
                        rust_files_paths[2].clone(),
                        rust_files_paths[3].clone(),
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<HashSet<PathBuf>>(),
                },
                TestCase {
                    dir_path: &dir_root_a,
                    expected: vec![rust_files_paths[1].clone(), rust_files_paths[2].clone()]
                        .into_iter()
                        .flatten()
                        .collect::<HashSet<PathBuf>>(),
                },
                TestCase {
                    dir_path: &dir_root_a_b,
                    expected: rust_files_paths[2].clone(),
                },
                TestCase {
                    dir_path: &dir_root_b,
                    expected: rust_files_paths[3].clone(),
                },
                TestCase {
                    dir_path: &dir_root_b_b,
                    expected: HashSet::new(),
                },
            ];

            for case in test_cases {
                let source_item = get_all_rs_files_in_dir(case.dir_path.to_str().unwrap());
                assert!(source_item.iter().all(|item| case.expected.contains(item)));
            }
        }
    }

    mod list_target_sources {
        use std::collections::HashSet;
        use std::fs;

        use cargo_metadata::camino::Utf8PathBuf;

        use bsp_types::requests::SourceItem;

        use crate::project_model::sources::{create_source_item, list_target_sources};
        use crate::project_model::target_details::{CargoTargetKind, TargetDetails};

        use super::*;

        struct TestCase<'a> {
            test_target_kind: CargoTargetKind,
            expected: &'a HashSet<SourceItem>,
        }

        #[test]
        fn test() {
            let root_dir1 = tempdir().unwrap();
            let root_dir1_path = root_dir1.path();
            let root_dir1_path_str = Utf8PathBuf::try_from(root_dir1_path.to_path_buf()).unwrap();

            let root_dir2 = tempdir().unwrap();
            let root_dir2_path = root_dir2.path();
            let root_dir2_path_str = Utf8PathBuf::try_from(root_dir2_path.to_path_buf()).unwrap();

            let src_dir_path = root_dir1_path.join("src");
            fs::create_dir(src_dir_path.clone()).unwrap();
            let tests_dir_path = root_dir1_path.join("tests");
            fs::create_dir(tests_dir_path.clone()).unwrap();

            let rust_source_files: Vec<HashSet<SourceItem>> =
                create_files_in_dirs(&RUST_FILE_NAMES, vec![&src_dir_path, &tests_dir_path])
                    .into_iter()
                    .map(|paths| {
                        paths
                            .iter()
                            .map(|path| create_source_item(path.clone()))
                            .collect()
                    })
                    .collect();

            let all_rust_source_files: HashSet<SourceItem> =
                rust_source_files.clone().into_iter().flatten().collect();

            let binding = Default::default();
            let default_target_details = TargetDetails {
                default_features_disabled: false,
                enabled_features: binding,
                ..Default::default()
            };

            let test_cases = vec![
                TestCase {
                    test_target_kind: CargoTargetKind::Bin,
                    expected: &rust_source_files[0],
                },
                TestCase {
                    test_target_kind: CargoTargetKind::Lib,
                    expected: &rust_source_files[0],
                },
                TestCase {
                    test_target_kind: CargoTargetKind::Test,
                    expected: &all_rust_source_files,
                },
                TestCase {
                    test_target_kind: CargoTargetKind::Bench,
                    expected: &all_rust_source_files,
                },
                TestCase {
                    test_target_kind: CargoTargetKind::Example,
                    expected: &all_rust_source_files,
                },
            ];

            for case in test_cases {
                let mut test_target_details = TargetDetails {
                    package_abs_path: root_dir1_path_str.clone(),
                    kind: case.test_target_kind.clone(),
                    ..default_target_details.clone()
                };

                let source_item = list_target_sources(test_target_details.clone());
                assert!(source_item.iter().all(|item| case.expected.contains(item)));

                test_target_details.package_abs_path = root_dir2_path_str.clone();
                let source_item = list_target_sources(test_target_details);
                assert_eq!(source_item.len(), 0);
            }
        }
    }

    #[test]
    fn get_sources_for_target_test() {
        let test_id = BuildTargetIdentifier {
            uri: "testId".to_string(),
        };
        let binding = Default::default();
        let test_target_details = TargetDetails {
            name: String::default(),
            kind: CargoTargetKind::Test,
            package_abs_path: Utf8PathBuf::from("/test_project_path"),
            package_name: Default::default(),
            default_features_disabled: false,
            enabled_features: binding,
        };

        assert_json_snapshot!(
            get_sources_for_target(&test_id, test_target_details),
            @r###"
        {
          "target": {
            "uri": "testId"
          },
          "sources": [],
          "roots": [
            "file:///test_project_path"
          ]
        }
        "###
        );
    }
}
