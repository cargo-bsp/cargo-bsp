use cargo_metadata::{Dependency, Package};
use std::path::PathBuf;

pub struct PackageWithDependencies<'a>(pub &'a Package, pub Vec<PackageDependency>);

pub struct PackageDependency {
    pub manifest_path: PathBuf,
    pub _features: Vec<String>,
    pub _uses_default_features: bool,
}

impl PackageDependency {
    pub fn new(dependency: &Dependency, all_packages: &[Package]) -> Self {
        let manifest_path = &all_packages
            .iter()
            .find(|p| p.name == dependency.name)
            .unwrap()
            .manifest_path;

        Self {
            manifest_path: manifest_path.into(),
            _features: dependency.features.clone(),
            _uses_default_features: dependency.uses_default_features,
        }
    }
}
