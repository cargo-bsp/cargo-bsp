// copy from rust-analyzer

use std::{
    fs::{self, read_dir, ReadDir},
    io,
};
use std::path::PathBuf;

use anyhow::{bail, format_err, Result};
use rustc_hash::FxHashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ManifestPath {
    file: PathBuf,
}

impl TryFrom<PathBuf> for ManifestPath {
    type Error = PathBuf;

    fn try_from(file: PathBuf) -> Result<Self, Self::Error> {
        if file.parent().is_none() {
            Err(file)
        } else {
            Ok(ManifestPath { file })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum ProjectManifest {
    CargoToml(ManifestPath),
}

impl ProjectManifest {
    pub fn from_manifest_file(path: PathBuf) -> Result<ProjectManifest> {
        let path = ManifestPath::try_from(path)
            .map_err(|path| format_err!("bad manifest path: {}", path.display()))?;
        if path.file.file_name().unwrap_or_default() == "Cargo.toml" {
            return Ok(ProjectManifest::CargoToml(path));
        }
        bail!("project root must point to Cargo.toml {}", path.file.display());
    }

    // TODO check how it works when cargo.toml not only in the the main folder
    pub fn discover(path: &PathBuf) -> io::Result<Vec<ProjectManifest>> {
        return find_cargo_toml(path)
            .map(|paths| paths.into_iter().map(ProjectManifest::CargoToml).collect());

        fn find_cargo_toml(path: &PathBuf) -> io::Result<Vec<ManifestPath>> {
            match find_in_parent_dirs(path, "Cargo.toml") {
                Some(it) => Ok(vec![it]),
                None => Ok(find_cargo_toml_in_child_dir(read_dir(path)?)),
            }
        }

        fn find_in_parent_dirs(path: &PathBuf, target_file_name: &str) -> Option<ManifestPath> {
            if path.file_name().unwrap_or_default() == target_file_name {
                if let Ok(manifest) = ManifestPath::try_from(path.to_path_buf()) {
                    return Some(manifest);
                }
            }

            let mut curr = Some(path.to_path_buf());

            while let Some(path) = curr {
                let candidate = path.join(target_file_name);
                if fs::metadata(&candidate).is_ok() {
                    if let Ok(manifest) = ManifestPath::try_from(candidate) {
                        return Some(manifest);
                    }
                }
                curr = path.parent().map(PathBuf::from);
            }

            None
        }

        fn find_cargo_toml_in_child_dir(entities: ReadDir) -> Vec<ManifestPath> {
            entities
                .filter_map(Result::ok)
                .map(|it| it.path().join("Cargo.toml"))
                .filter(|it| it.exists())
                .filter_map(|it| it.try_into().ok())
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