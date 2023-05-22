use std::path::PathBuf;
use std::sync::Arc;

use cargo_metadata::camino::Utf8PathBuf;
use log::warn;
use walkdir::WalkDir;

use bsp_types::requests::{SourceItem, SourceItemKind, SourcesItem};
use bsp_types::BuildTargetIdentifier;

use crate::project_model::target_details::{CargoTargetKind, TargetDetails};
use crate::project_model::workspace::ProjectWorkspace;
use crate::utils::uri::file_uri;

pub fn get_sources_item(
    workspace: &Arc<ProjectWorkspace>,
    id: BuildTargetIdentifier,
) -> Option<SourcesItem> {
    let target_details = workspace.get_target_details(&id).or_else(|| {
        warn!("Failed to get target details for: {:?}", id);
        None
    })?;
    let package_path = target_details.package_abs_path.clone();

    Some(SourcesItem {
        target: id,
        sources: list_target_sources(target_details),
        roots: vec![file_uri(package_path)],
    })
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
        _ => src_sources.append(&mut list_source_files_in_path(package_path.join("tests"))),
    }

    src_sources
}

fn get_all_rs_files_in_dir(dir: &str) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let f_name = e.file_name().to_string_lossy();
                if f_name.ends_with(".rs") {
                    Some(e.into_path())
                } else {
                    None
                }
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
    use std::fs::File;
    use std::path::PathBuf;

    use tempfile::{tempdir, tempdir_in, TempDir};

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
            let temp_file = File::create(temp_file_path.clone()).unwrap();

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

            drop(temp_file);
            temp_dir.close().unwrap();
        }
    }

    mod get_all_rs_files_in_dir_test {
        use std::collections::HashSet;
        use std::path::Path;

        use crate::project_model::sources::get_all_rs_files_in_dir;

        use super::*;

        struct TestCase {
            dir_path: String,
            expected: HashSet<PathBuf>,
        }

        const RUST_FILE_NAMES: [&str; 3] = ["test1.rs", "test2.rs", "test3.rs"];
        const NOT_RUST_FILE_NAMES: [&str; 3] = ["test1.txt", "test4", "test5.rs.java"];

        fn create_files(files_names: &[&str], dir: &Path) -> (HashSet<PathBuf>, Vec<File>) {
            let files_paths = files_names
                .iter()
                .map(|name| dir.join(name))
                .collect::<Vec<PathBuf>>();
            let files = files_paths
                .iter()
                .map(|path| File::create(path.clone()).unwrap())
                .collect::<Vec<File>>();

            (files_paths.into_iter().collect(), files)
        }

        fn create_files_in_dirs(
            files_names: &[&str],
            dirs: &[TempDir],
        ) -> (Vec<HashSet<PathBuf>>, Vec<Vec<File>>) {
            dirs.iter()
                .map(|dir| create_files(files_names, dir.path()))
                .unzip()
        }

        fn create_dirs() -> Vec<TempDir> {
            let dir1 = tempdir().unwrap();
            let dir2 = tempdir_in(dir1.path()).unwrap();
            let dir3 = tempdir_in(dir2.path()).unwrap();
            let dir4 = tempdir_in(dir1.path()).unwrap();
            let dir5 = tempdir_in(dir4.path()).unwrap();

            vec![dir1, dir2, dir3, dir4, dir5]
        }

        #[test]
        fn test() {
            let dirs = create_dirs();

            let (rust_files_paths, rust_files) =
                create_files_in_dirs(&RUST_FILE_NAMES, &dirs.as_slice()[..=3]);
            let (_, not_rust_files) =
                create_files_in_dirs(&NOT_RUST_FILE_NAMES, &dirs.as_slice()[3..]);

            let test_cases = vec![
                TestCase {
                    dir_path: dirs[0].path().to_str().unwrap().to_string(),
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
                    dir_path: dirs[1].path().to_str().unwrap().to_string(),
                    expected: vec![rust_files_paths[1].clone(), rust_files_paths[2].clone()]
                        .into_iter()
                        .flatten()
                        .collect::<HashSet<PathBuf>>(),
                },
                TestCase {
                    dir_path: dirs[2].path().to_str().unwrap().to_string(),
                    expected: rust_files_paths[2].clone(),
                },
                TestCase {
                    dir_path: dirs[3].path().to_str().unwrap().to_string(),
                    expected: rust_files_paths[3].clone(),
                },
                TestCase {
                    dir_path: dirs[4].path().to_str().unwrap().to_string(),
                    expected: HashSet::new(),
                },
            ];

            for case in test_cases {
                let source_item = get_all_rs_files_in_dir(&case.dir_path);
                assert!(source_item.iter().all(|item| case.expected.contains(item)));
            }

            rust_files.into_iter().for_each(drop);
            not_rust_files.into_iter().for_each(drop);
            dirs.into_iter().rev().for_each(|dir| dir.close().unwrap());
        }
    }
}
