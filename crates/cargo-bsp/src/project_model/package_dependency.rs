//! Stores package dependencies.

use std::path::PathBuf;

use cargo_metadata::{Dependency, Package};
use log::error;

use bsp_types::BuildTargetIdentifier;

use crate::utils::uri::file_uri;

#[derive(Default, Debug, Clone)]
pub struct PackageDependency {
    /// Dependency name
    pub name: String,
    /// Path to the dependency's manifest
    pub manifest_path: PathBuf,
    /// Whether this dependency is optional and needs to be enabled by feature
    pub optional: bool,
    /// Features which are enabled for this dependency
    pub _features: Vec<String>,
    /// Whether this dependency uses the default features
    pub _uses_default_features: bool,
}

impl PackageDependency {
    pub fn new(dependency: &Dependency, all_packages: &[Package]) -> Option<Self> {
        all_packages
            .iter()
            .find(|p| p.name == dependency.name)
            .map(|p| Self {
                name: dependency.name.clone(),
                manifest_path: p.manifest_path.clone().into(),
                optional: dependency.optional,
                _features: dependency.features.clone(),
                _uses_default_features: dependency.uses_default_features,
            })
            .or_else(|| {
                error!("Failed to find package with name: {}", dependency.name);
                None
            })
    }

    pub fn create_package_dependencies_from_metadata(
        metadata_dependencies: &[Dependency],
        all_packages: &[Package],
    ) -> Vec<PackageDependency> {
        metadata_dependencies
            .iter()
            .filter_map(|dep| PackageDependency::new(dep, all_packages))
            .collect()
    }

    pub fn create_id_from_dependency(&self) -> Option<BuildTargetIdentifier> {
        if let Some(manifest_path_str) = self.manifest_path.to_str() {
            Some(BuildTargetIdentifier {
                uri: file_uri(manifest_path_str),
            })
        } else {
            error!(
                "Failed extracting manifest path from dependency: {:?}",
                self.manifest_path
            );
            None
        }
    }
}
