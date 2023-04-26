use crate::bsp_types::mappings::file_uri;
use crate::bsp_types::BuildTargetIdentifier;
use cargo_metadata::{Dependency, Package};
use log::error;
use std::path::PathBuf;

pub struct PackageWithDependenciesIds<'a>(pub &'a Package, pub Vec<BuildTargetIdentifier>);

#[derive(Default, Debug)]
pub struct PackageDependency {
    /// Path to the dependency's manifest
    pub manifest_path: PathBuf,
    // Whether this dependency is optional and need to be enabled by feature
    pub optional: bool,
    /// Features which are enabled for this dependency
    pub _features: Vec<String>,
    /// whether this dependency uses the default features
    pub _uses_default_features: bool,
}

impl PackageDependency {
    pub fn new(dependency: &Dependency, all_packages: &[Package]) -> Option<Self> {
        all_packages
            .iter()
            .find(|p| p.name == dependency.name)
            .map(|p| Self {
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

    pub fn map_from_metadata_dependencies(
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

/* Optional dependencies

-check if Metadata.Package.Dependecies.optional == True?
  -check if feature is enabled*
    -if yes, add to dependencies
    -if no, skip

* how to do it?
- store features in a struct vector
    - store in feature: enabled, to which is mapped, is_default?
    - store in package default features?
- retrieve them from cargo metadata and set defaults
- implement methods for no-default

** how to check if feature is enabled?
 - check if feature is package_name/feature_name
 - cjeck if feature is package_name
 - check if feature is dep:package_name
*/
