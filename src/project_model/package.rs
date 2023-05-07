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

    /// List of enabled (by BSP client) features.
    /// Does not include default features.
    pub enabled_features: Vec<String>,

    /// If true, default features are disabled. Does not apply when default features
    /// are not defined in package's manifest.
    pub default_features_disabled: bool,

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
            enabled_features: vec![],
            default_features_disabled: false,
            package_features: metadata_package.features.clone(),
        }
    }

    /// We assume that optional dependency can only be turned on by a feature that has the form:
    /// "dep:package_name" or "package_name/feature_name"
    fn check_if_enabling_feature(feature: &str, dependency_name: &String) -> bool {
        feature == format!("dep:{}", dependency_name)
            || feature.starts_with(&format!("{}/", dependency_name))
    }

    /// Checks if a feature was defined in the Cargo.toml Used to skip features that have the form:
    /// "dep:package_name" or "package_name/feature_name" or "package_name?/feature_name" as they
    /// are not included in the cargo metadata features Hashmap
    fn is_defined_feature(&self, feature: &String) -> bool {
        self.package_features.contains_key(feature)
    }

    /// Checks whether a dependency is enabled by the current set of enabled features.
    fn is_dependency_enabled(&self, dependency: &PackageDependency) -> bool {
        if !dependency.optional {
            return true;
        }

        let mut next_features = VecDeque::from(self.enabled_features.clone());
        if !self.default_features_disabled && self.package_features.contains_key("default") {
            next_features.push_back(String::from("default"));
        }

        let mut checked_features: HashSet<String> =
            HashSet::from_iter(next_features.iter().cloned());

        while let Some(f) = next_features.pop_front() {
            let dependent_features = self.package_features.get(&f);
            if dependent_features.is_none() {
                error!("Feature {} not found in package {}", f, self.name);
                continue;
            }

            for df in dependent_features.unwrap() {
                if BspPackage::check_if_enabling_feature(df, &dependency.name) {
                    // Feature is enabling and so dependency is enabled
                    return true;
                }
                if checked_features.contains(df) || self.is_defined_feature(df) {
                    continue;
                }
                checked_features.insert(df.clone());
                next_features.push_back(df.clone());
            }
        }
        false
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
