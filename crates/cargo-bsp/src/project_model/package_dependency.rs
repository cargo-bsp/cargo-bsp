//! Represents a dependency of a package.
//! Fields `features` and `uses_default_features` may become handy when implementing
//! `BuildTargetDependencyModules` request. Optional dependencies of the `PackageDependency` are
//! included only if specific features for `PackageDependency` are set.
//! Similarly, disabling default features impact the set of dependencies of the `PackageDependency`.
//!
//! Currently fields: `features`, `uses_default_features`, `rename`, `kind` and `target`
//! are used only to handle the BSP Rust extension.

use std::path::PathBuf;

use cargo_metadata::{Dependency, DependencyKind, Package};
use cargo_platform::Platform;
use log::error;

use bsp_types::{extensions::Feature, BuildTargetIdentifier};

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
    pub features: Vec<Feature>,
    /// Whether this dependency uses the default features
    pub uses_default_features: bool,
    /// Name to which this dependency is renamed when declared in Cargo.toml
    pub rename: Option<String>,
    /// The kind of the dependency (normal, build, dev)
    pub kind: DependencyKind,
    /// The target platform for the dependency. None if not a target dependency.
    pub target: Option<Platform>,
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
                features: dependency
                    .features
                    .iter()
                    .map(|f| Feature(f.clone()))
                    .collect(),
                uses_default_features: dependency.uses_default_features,
                rename: dependency.rename.clone(),
                kind: dependency.kind,
                target: dependency.target.clone(),
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
