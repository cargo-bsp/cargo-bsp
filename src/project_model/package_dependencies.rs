use crate::bsp_types::mappings::file_uri;
use crate::bsp_types::BuildTargetIdentifier;
use cargo_metadata::{Dependency, Package};
use log::error;
use std::path::PathBuf;

pub struct PackageWithDependenciesIds<'a>(pub &'a Package, pub Vec<BuildTargetIdentifier>);

pub struct PackageDependency {
    pub manifest_path: PathBuf,
    pub _features: Vec<String>,
    pub _uses_default_features: bool,
}

impl PackageDependency {
    pub fn new(dependency: &Dependency, all_packages: &[Package]) -> Option<Self> {
        all_packages
            .iter()
            .find(|p| p.name == dependency.name)
            .map(|p| Self {
                manifest_path: p.manifest_path.clone().into(),
                _features: dependency.features.clone(),
                _uses_default_features: dependency.uses_default_features,
            })
            .or_else(|| {
                error!("Failed to find package with name: {}", dependency.name);
                None
            })
    }

    pub fn create_id_from_dependency(&self) -> Option<BuildTargetIdentifier> {
        // TODO: take into account features - maybe can be set with cargo metadata
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
