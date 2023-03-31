// copy from rust-analyzer

use std::path::{Path, PathBuf};
use std::{
    fs::{self, read_dir, ReadDir},
    io,
};

use anyhow::Result;
use rustc_hash::FxHashSet;

use crate::logger::log;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Default)]
pub struct ProjectManifest {
    pub file: PathBuf,
}

impl ProjectManifest {
    pub fn discover_all(path: &PathBuf) -> io::Result<Vec<ProjectManifest>> {
        return find_cargo_toml(path).map(|paths| {
            paths
                .into_iter()
                .map(|val| ProjectManifest { file: val })
                .collect()
        });

        fn valid_path(file: PathBuf) -> Result<PathBuf, PathBuf> {
            if file.parent().is_none() {
                Err(file)
            } else {
                Ok(file)
            }
        }

        fn find_cargo_toml(path: &PathBuf) -> io::Result<Vec<PathBuf>> {
            match find_in_parent_dirs(path) {
                Some(it) => Ok(vec![it]),
                None => Ok(find_cargo_toml_in_child_dir(read_dir(path)?)),
            }
        }

        fn find_in_parent_dirs(path: &Path) -> Option<PathBuf> {
            if path.file_name().unwrap_or_default() == "Cargo.toml" {
                if let Ok(path) = valid_path(path.to_path_buf()) {
                    return Some(path);
                }
            }

            let mut curr = Some(path.to_path_buf());

            while let Some(path) = curr {
                let candidate = path.join("Cargo.toml");
                if fs::metadata(&candidate).is_ok() {
                    if let Ok(manifest) = valid_path(candidate) {
                        return Some(manifest);
                    }
                }
                curr = path.parent().map(PathBuf::from);
            }

            None
        }

        fn find_cargo_toml_in_child_dir(entities: ReadDir) -> Vec<PathBuf> {
            entities
                .filter_map(Result::ok)
                .map(|it| it.path().join("Cargo.toml"))
                .filter(|it| it.exists())
                .collect()
        }
    }

    pub fn discover(path: &PathBuf) -> Result<ProjectManifest, &'static str> {
        let res = ProjectManifest::discover_all(path)
            .unwrap_or_default()
            .into_iter()
            .collect::<FxHashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        match res.len() {
            0 => Err("Cargo.toml not found"),
            x => {
                if x != 1 {
                    log(&format!(
                        "warning: Discovered more than one workspace, proceeding with {:?}",
                        res[0]
                    ));
                }
                Ok(res[0].clone())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::fs::File;
    use std::path::PathBuf;

    use tempfile::{tempdir, tempdir_in};

    use crate::project_model::ProjectManifest;

    struct TestCase<'a> {
        path: &'a PathBuf,
        at_least_one: bool,
        result: &'a HashSet<ProjectManifest>,
    }

    fn test_paths(cases: &Vec<TestCase>) {
        for case in cases {
            let result = ProjectManifest::discover_all(case.path);
            assert!(result
                .unwrap()
                .iter()
                .all(|item| case.result.contains(item)));

            let result = ProjectManifest::discover(case.path);
            if case.at_least_one {
                assert!(case.result.contains(&result.unwrap()));
            } else {
                assert_eq!(result.unwrap_err(), "Cargo.toml not found");
            }
        }
    }

    #[test]
    fn no_toml() {
        let main_dir = tempdir().unwrap();
        let inner_dir1 = tempdir_in(main_dir.path()).unwrap();
        let inner_dir2 = tempdir_in(main_dir.path()).unwrap();
        let inner_inner_dir = tempdir_in(inner_dir2.path()).unwrap();

        let file_path = inner_inner_dir.path().join("Cargo.toml");
        let file = File::create(file_path).unwrap();

        test_paths(&vec![
            TestCase {
                path: &main_dir.path().to_path_buf(),
                at_least_one: false,
                result: &HashSet::new(),
            },
            TestCase {
                path: &inner_dir1.path().to_path_buf(),
                at_least_one: false,
                result: &HashSet::new(),
            },
        ]);

        drop(file);
        inner_inner_dir.close().unwrap();
        inner_dir2.close().unwrap();
        inner_dir1.close().unwrap();
        main_dir.close().unwrap();
    }

    #[test]
    fn one_toml() {
        let main_dir = tempdir().unwrap();
        let inner_dir = tempdir_in(main_dir.path()).unwrap();

        let file_path = inner_dir.path().join("Cargo.toml");
        let file = File::create(file_path.clone()).unwrap();

        let expected = HashSet::from([ProjectManifest {
            file: file_path.clone(),
        }]);

        test_paths(&vec![
            TestCase {
                path: &PathBuf::from("Cargo.toml"),
                at_least_one: true,
                result: &HashSet::from([ProjectManifest {
                    file: PathBuf::from("Cargo.toml"),
                }]),
            },
            TestCase {
                path: &file_path,
                at_least_one: true,
                result: &expected,
            },
            TestCase {
                path: &inner_dir.path().to_path_buf(),
                at_least_one: true,
                result: &expected,
            },
            TestCase {
                path: &main_dir.path().to_path_buf(),
                at_least_one: true,
                result: &expected,
            },
        ]);

        drop(file);
        inner_dir.close().unwrap();
        main_dir.close().unwrap();
    }

    #[test]
    fn more_than_one_toml() {
        let main_dir = tempdir().unwrap();
        let inner_dir1 = tempdir_in(main_dir.path()).unwrap();
        let inner_dir2 = tempdir_in(main_dir.path()).unwrap();

        let file_path1 = inner_dir1.path().join("Cargo.toml");
        let file1 = File::create(file_path1.clone()).unwrap();

        let file_path2 = inner_dir2.path().join("Cargo.toml");
        let file2 = File::create(file_path2.clone()).unwrap();

        test_paths(&vec![TestCase {
            path: &main_dir.path().to_path_buf(),
            at_least_one: true,
            result: &HashSet::from([
                ProjectManifest { file: file_path1 },
                ProjectManifest { file: file_path2 },
            ]),
        }]);

        drop(file1);
        drop(file2);
        inner_dir1.close().unwrap();
        inner_dir2.close().unwrap();
        main_dir.close().unwrap();
    }
}
