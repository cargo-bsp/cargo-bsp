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

    let package_path = target_details.package_abs_path.clone();

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

    use tempfile::tempdir;

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

        const RUST_FILE_NAMES: [&str; 3] = ["test1.rs", "test2.rs", "test3.rs"];
        const NOT_RUST_FILE_NAMES: [&str; 3] = ["test1.txt", "test4", "test5.rs.java"];

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


}
