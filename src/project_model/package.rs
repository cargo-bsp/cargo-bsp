use crate::bsp_types::mappings::build_target::bsp_build_target_from_cargo_target;
use crate::bsp_types::{BuildTarget, BuildTargetIdentifier};
use crate::project_model::package_dependencies::PackageDependency;
use cargo_metadata::camino::Utf8PathBuf;
use log::error;
use std::collections::{HashMap, HashSet, VecDeque};
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
    /// Does not include default features
    pub enabled_features: Vec<String>,

    /// If true, default features are disabled. Does not apply
    /// when default features are not defined in Cargo.toml
    pub no_default_features: bool,

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
            targets: metadata_package
                .targets
                .iter()
                .cloned()
                .map(Rc::new)
                .collect(),
            enabled_features: vec![], //todo check default
            no_default_features: false,
            package_features: metadata_package.features.clone(),
        }
    }

    fn is_dependency_enabled(&self, dependency: &PackageDependency) -> bool {
        if !dependency.optional {
            return true;
        }

        let mut next_features = VecDeque::from(self.enabled_features.clone());
        if self.no_default_features {
            next_features.push_back(String::from("default"));
        }

        let mut checked_features = HashSet::new();
        checked_features.extend(next_features.clone());

        while let Some(f) = next_features.pop_front() {
            if let Some(dependent_features) = self.package_features.get(&f) {


                dependent_features
                    .iter()
                    .for_each(|f| {
                        if checked_features.contains(f) {
                            return;
                        }
                        checked_features.insert(f.clone());
                        next_features.push_back(f.clone());
                    });
            } else {
                error!("Feature {} not found in package {}", f, self.name);
            }
        }

        self.enabled_features.iter().for_each(|f| {
            let dependent_features = self.package_features.get(f).or_else(|| {
                error!("Feature {} not found in package {}", f, self.name);
                None
            });

            todo!("bfs through dependent_features, check the current")
        });

        todo!("BFS dep:, package_name, package_name/feature_name");
        true
    }

    fn feature_based_dependencies_as_build_target_ids(&self) -> Vec<BuildTargetIdentifier> {
        self.dependencies
            .iter()
            .filter_map(|dep| {
                if !self.is_dependency_enabled(dep) {
                    return None;
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
        features.iter().for_each(|f| {
            if self.package_features.get(f).is_none() || self.enabled_features.contains(f) {
                error!("Feature {} doesn't exist or is already enabled.", f);
                return;
            }
            self.enabled_features.push(f.clone())
        });
    }

    pub fn disable_features(&mut self, features: &[String]) {
        features.iter().for_each(|f| {
            if self.package_features.get(f).is_none() || !self.enabled_features.contains(f) {
                error!("Feature {} doesn't exist or isn't enabled.", f);
                return;
            }
            self.enabled_features.retain(|x| x != f);
        });
    }

    /// Returns list of dependencies taking into account optional ones and enabled features
    pub fn get_enabled_dependencies(&self) -> Vec<&PackageDependency> {
        self.dependencies
            .iter()
            .filter(|&d| self.is_dependency_enabled(d))
            .collect()
    }
}
