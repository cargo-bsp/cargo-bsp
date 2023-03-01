// copy from rust-analyzer

use std::{
    fs::{self, read_dir, ReadDir},
    io,
};
use std::path::{Path, PathBuf};

use anyhow::Result;
use rustc_hash::FxHashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ProjectManifest {
    pub file: PathBuf,
}

impl ProjectManifest {
    pub fn discover(path: &PathBuf) -> io::Result<Vec<ProjectManifest>> {
        return find_cargo_toml(path)
            .map(|paths| paths.into_iter().map(|val| ProjectManifest { file: val }).collect());


        fn valid_path(file: PathBuf) -> Result<PathBuf, PathBuf> {
            if file.parent().is_none() {
                Err(file)
            } else {
                Ok(file)
            }
        }

        fn find_cargo_toml(path: &PathBuf) -> io::Result<Vec<PathBuf>> {
            match find_in_parent_dirs(path, "Cargo.toml") {
                Some(it) => Ok(vec![it]),
                None => Ok(find_cargo_toml_in_child_dir(read_dir(path)?)),
            }
        }

        fn find_in_parent_dirs(path: &Path, target_file_name: &str) -> Option<PathBuf> {
            if path.file_name().unwrap_or_default() == target_file_name {
                if let Ok(path) = valid_path(path.to_path_buf()) {
                    return Some(path);
                }
            }

            let mut curr = Some(path.to_path_buf());

            while let Some(path) = curr {
                let candidate = path.join(target_file_name);
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

    pub fn discover_all(path: &PathBuf) -> Vec<ProjectManifest> {
        let mut res = ProjectManifest::discover(path)
            .unwrap_or_default()
            .into_iter()
            .collect::<FxHashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        res.sort();
        res
    }


}


