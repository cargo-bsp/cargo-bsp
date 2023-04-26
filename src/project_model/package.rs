use crate::bsp_types::mappings::build_target::bsp_build_target_from_cargo_target;
use crate::bsp_types::{BuildTarget, BuildTargetIdentifier};
use crate::project_model::package_dependencies::PackageDependency;
use cargo_metadata::camino::Utf8PathBuf;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default, Debug)]
pub struct BspPackage {
    /// Name of the package
    pub name: String,

    /// Path to the package's manifest
    pub manifest_path: Utf8PathBuf,

    /// List of all package dependencies
    pub dependencies: Vec<PackageDependency>,

    /// List of all package targets, from which BSP build targets are created
    pub targets: Vec<Rc<cargo_metadata::Target>>,

    /// List of enabled in package features
    pub enabled_features: Vec<String>,

    /// Hashmap where key is a feature name and the value are names of other features it enables.
    /// Includes pair for default if defined.
    pub package_features: HashMap<String, Vec<String>>,
}

impl BspPackage {
    pub fn new(
        metadata_package: &cargo_metadata::Package,
        all_packages: &[cargo_metadata::Package],
    ) -> Self {
        Self {
            name: metadata_package.name.clone(),
            manifest_path: metadata_package.manifest_path.clone(),
            dependencies: PackageDependency::map_from_metadata_dependencies(
                &metadata_package.dependencies,
                all_packages,
            ),
            targets: metadata_package.targets.iter().cloned().map(Rc::new).collect(),
            enabled_features: vec![], //todo check default
            package_features: metadata_package.features.clone(),
        }
    }

    fn feature_based_dependencies_as_build_target_ids(&self) -> Vec<BuildTargetIdentifier> {
        self.dependencies
            .iter()
            .filter_map(|dep| {
                if dep.optional {
                    todo!("dep:, package_name, package_name/feature_name");
                }

                dep.create_id_from_dependency()
            })
            .collect()
    }

    pub fn get_bsp_build_targets(&self) -> Vec<BuildTarget> {
        let dependencies = self.feature_based_dependencies_as_build_target_ids();
        self.targets
            .iter()
            .map(|t| bsp_build_target_from_cargo_target(t, &dependencies))
            .collect()
    }

    pub fn enable_features(&mut self, features: &[String]) {
        for feature in features {
            todo!("BFS");
            //todo check if feature exists
            //todo check if feature is already enabled
        }
    }

    pub fn disable_features(&mut self, features: &[String]) {
        for feature in features {
            todo!("BFS");
            //todo check if feature exists
            //todo check if feature is already enabled
        }
    }

    /// Returns list of dependencies taking into account optional ones and enabled features
    pub fn get_enabled_dependencies(&self) -> Vec<PackageDependency> {
        todo!()
    }
}
