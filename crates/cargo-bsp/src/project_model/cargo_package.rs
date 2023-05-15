use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::hash::Hash;
use std::rc::Rc;

use cargo_metadata::camino::Utf8PathBuf;
use log::{error, warn};

use bsp_types::{BuildTarget, BuildTargetIdentifier};

use crate::project_model::build_target_mappings::bsp_build_target_from_cargo_target;
use crate::project_model::package_dependency::PackageDependency;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Feature(String);

impl From<&str> for Feature {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Default, Debug)]
pub struct CargoPackage {
    /// Name of the package
    pub name: String,

    /// Path to the package's manifest
    pub manifest_path: Utf8PathBuf,

    /// List of all package dependencies
    pub dependencies: Vec<PackageDependency>,

    /// List of all package targets, from which BSP build targets are created
    pub targets: Vec<Rc<cargo_metadata::Target>>,

    /// List of enabled (by BSP client) features.
    /// Does not include default features
    pub enabled_features: BTreeSet<Feature>,

    /// If true, default features are disabled. Does not apply when default features
    /// are not defined in package's manifest
    pub default_features_disabled: bool,

    /// Hashmap where key is a feature name and the value are names of other features it enables.
    /// Includes pair for default features if default is defined
    pub package_features: BTreeMap<Feature, Vec<Feature>>,
}

impl CargoPackage {
    pub fn new(
        metadata_package: &cargo_metadata::Package,
        all_packages: &[cargo_metadata::Package],
    ) -> Self {
        let package_features = metadata_package
            .features
            .clone()
            .into_iter()
            .map(|(f, df)| (Feature(f), df.into_iter().map(Feature).collect()))
            .collect();

        Self {
            name: metadata_package.name.clone(),
            manifest_path: metadata_package.manifest_path.clone(),
            dependencies: PackageDependency::create_package_dependencies_from_metadata(
                &metadata_package.dependencies,
                all_packages,
            ),
            targets: metadata_package
                .targets
                .iter()
                .cloned()
                .map(Rc::new)
                .collect(),
            enabled_features: BTreeSet::new(),
            default_features_disabled: false,
            package_features,
        }
    }

    /// We assume that optional dependency can only be turned on by a feature that has the form:
    /// "dep:package_name" or "package_name/feature_name"
    fn feature_enables_dependency(feature: &Feature, dependency_name: &String) -> bool {
        feature.0 == format!("dep:{}", dependency_name)
            || feature.0.starts_with(&format!("{}/", dependency_name))
    }

    /// Checks if a feature was defined in the `Cargo.toml`. Used to skip features that have the form:
    /// "dep:package_name" or "package_name/feature_name" or "package_name?/feature_name" as they
    /// are not included in the cargo metadata features Hashmap
    fn is_defined_feature(&self, feature: &Feature) -> bool {
        self.package_features.contains_key(feature)
    }

    /// Checks whether a dependency is enabled by the current set of enabled features.
    /// Runs BFS on the features graph starting from default (if defined and not disabled)
    /// and the enabled features
    fn is_dependency_enabled(&self, dependency: &PackageDependency) -> bool {
        if !dependency.optional {
            return true;
        }

        let mut next_features: VecDeque<Feature> =
            VecDeque::from_iter(self.enabled_features.clone());
        if !self.default_features_disabled && self.is_defined_feature(&Feature::from("default")) {
            next_features.push_back(Feature::from("default"));
        }

        let mut checked_features: HashSet<Feature> = HashSet::from_iter(next_features.clone());

        while let Some(f) = next_features.pop_front() {
            if let Some(dependent_features) = self.package_features.get(&f) {
                for df in dependent_features {
                    if CargoPackage::feature_enables_dependency(df, &dependency.name) {
                        return true;
                    }
                    if checked_features.contains(df) || !self.is_defined_feature(df) {
                        continue;
                    }
                    checked_features.insert(df.clone());
                    next_features.push_back(df.clone());
                }
            } else {
                error!("Feature {:?} not found in package {}", f, self.name);
            }
        }
        false
    }

    /// Returns a vector of BuildTargetIdentifiers for all dependencies that
    /// * are enabled
    /// * their BuildTargetId could be created
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

    /// Returns a vector of BuildTargets for all targets in the package
    pub fn get_bsp_build_targets(&self) -> Vec<BuildTarget> {
        let dependencies = self.feature_based_dependencies_as_build_target_ids();
        self.targets
            .iter()
            .map(|t| bsp_build_target_from_cargo_target(t, &dependencies))
            .collect()
    }

    /// Enables a list of features if they exist and are not already enabled
    pub fn enable_features(&mut self, features: &[Feature]) {
        features.iter().for_each(|f| {
            if self.package_features.get(f).is_none() {
                warn!("Can't enable feature {:?}. It doesn't exist.", f);
                return;
            }
            self.enabled_features.insert(f.clone());
        });
    }

    /// Disables a list of features if they exist and are enabled
    pub fn disable_features(&mut self, features: &[Feature]) {
        features.iter().for_each(|f| {
            if self.package_features.get(f).is_none() {
                warn!("Can't disable feature {:?}. It doesn't exist.", f);
                return;
            }
            self.enabled_features.remove(f);
        });
    }

    /// Returns list of dependencies taking into account optional ones and enabled features
    pub fn _get_enabled_dependencies(&self) -> Vec<&PackageDependency> {
        self.dependencies
            .iter()
            .filter(|&d| self.is_dependency_enabled(d))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use test_case::test_case;

    use crate::project_model::cargo_package::{CargoPackage, Feature};

    const DEP_NAME: &str = "dependency-name";
    const F1: &str = "feature1";
    const F2: &str = "feature2";
    const F3: &str = "feature3";
    const F4: &str = "feature4";

    fn create_feature_set_from_slices(slices: &[&str]) -> BTreeSet<Feature> {
        slices.iter().map(|&f| Feature::from(f)).collect()
    }

    fn create_package_features(slice_map: &[(&str, &[&str])]) -> BTreeMap<Feature, Vec<Feature>> {
        slice_map
            .iter()
            .map(|&(k, v)| {
                (
                    Feature::from(k),
                    v.iter().map(|&s| Feature::from(s)).collect(),
                )
            })
            .collect()
    }

    fn default_cargo_package_with_features(
        package_features_slice: &[(&str, &[&str])],
        enabled_features_slice: Option<&[&str]>,
    ) -> CargoPackage {
        let mut test_package = CargoPackage {
            package_features: create_package_features(package_features_slice),
            ..CargoPackage::default()
        };

        if let Some(enabled_features_slice) = enabled_features_slice {
            test_package.enabled_features = create_feature_set_from_slices(enabled_features_slice);
        }

        test_package
    }

    #[test_case("feature-name", false ; "simple-feature")]
    #[test_case(DEP_NAME, false ; "dependency-name")]
    #[test_case(&format!("dep:{}", DEP_NAME), true ; "dep:dependency-name")]
    #[test_case(&format!("{}/feature", DEP_NAME), true ; "dependency-name/feature")]
    #[test_case(&format!("{}?/feature", DEP_NAME), false ; "dependency-name(question-mark)/feature")]
    fn test_feature_enables_dependency(feature: &str, expected_with_dep_name_in_feature: bool) {
        assert_eq!(
            expected_with_dep_name_in_feature,
            CargoPackage::feature_enables_dependency(
                &Feature::from(feature),
                &String::from(DEP_NAME)
            )
        );
        assert!(!CargoPackage::feature_enables_dependency(
            &Feature::from(feature),
            &String::from("other-dependency-name")
        ));
    }

    #[test_case(&[(F1, &[])], F1, true ; "just_one_feature_check_defined")]
    #[test_case(&[(F1, &[])], F2, false ; "just_one_feature_check_not_defined")]
    #[test_case(&[(F1, &[F2])], F2, false ; "one_feature_with_one_dependency_check_dependency")]
    #[test_case(&[(F1, &[F2]), (F2, &[F3]), (F3, &["dep:name"])], "dep:name", false ; "not_defined_in_many")]
    #[test_case(&[(F1, &[]), (F2, &[]), (F3, &[])], F3, true ; "defined_in_many")]
    fn test_is_defined_feature2(
        package_features_slice: &[(&str, &[&str])],
        feature: &str,
        expected: bool,
    ) {
        let test_package = default_cargo_package_with_features(package_features_slice, None);
        assert_eq!(
            expected,
            test_package.is_defined_feature(&Feature::from(feature))
        );
    }

    // enabling_features
    #[test_case(&[], &[], &[], &[], true ; "enabling_features::no_features")]
    #[test_case(&[], &[], &[F1], &[F1], true ; "enabling_features::no_toggling")]
    #[test_case(&[], &[F2], &[], &[], true ; "enabling_features::feature_not_defined")]
    #[test_case(&[F1], &[F2], &[F1], &[F1], true ; "enabling_features::feature_not_defined2")]
    #[test_case(&[F1], &[F1], &[], &[F1], true ; "enabling_features::enable_nothing_enabled")]
    #[test_case(&[F1, F2], &[F2], &[F1], &[F1, F2], true ; "enabling_features::enable_some_enabled")]
    #[test_case(&[F1], &[F1], &[F1], &[F1], true ; "enabling_features::enable_already_enabled")]
    #[test_case(&[F1, F2], &[F1, F2], &[F1], &[F1, F2], true ; "enabling_features::enable_many")]
    // disabling features
    #[test_case(&[], &[], &[], &[], false ; "disabling_features::no_features")]
    #[test_case(&[], &[], &[F1], &[F1], false ; "disabling_features::no_toggling")]
    #[test_case(&[], &[F1], &[], &[], false ; "disabling_features::feature_not_defined")]
    #[test_case(&[F1], &[F2], &[F1], &[F1], false ; "disabling_features::feature_not_defined2")]
    #[test_case(&[F1], &[F1], &[F1], &[], false ; "disabling_features::disable_one")]
    #[test_case(&[F1, F2], &[F2], &[F1], &[F1], false ; "disabling_features::disable_already_disabled")]
    #[test_case(&[F1, F2, F3], &[F2, F3], &[F1, F2, F3], &[F1], false ; "disabling_features::disable_many")]
    fn test_toggling_features(
        defined_features: &[&str],
        features_to_toggle: &[&str],
        enabled_features_slice: &[&str],
        expected: &[&str],
        test_enabling: bool,
    ) {
        let defined_features_map = defined_features
            .iter()
            .map(|&f| (f, &[] as &[&str]))
            .collect::<Vec<(&str, &[&str])>>();
        let mut test_package = default_cargo_package_with_features(
            &defined_features_map,
            Some(enabled_features_slice),
        );

        let expected = create_feature_set_from_slices(expected);
        let features_to_toggle: Vec<Feature> =
            features_to_toggle.iter().map(|&f| f.into()).collect();

        if test_enabling {
            test_package.enable_features(&features_to_toggle);
        } else {
            test_package.disable_features(&features_to_toggle);
        }

        assert_eq!(test_package.enabled_features, expected);
    }

    mod test_is_dependency_enabled {
        use ntest::timeout;
        use test_case::test_case;

        use crate::project_model::package_dependency::PackageDependency;

        use super::{default_cargo_package_with_features, DEP_NAME, F1, F2, F3, F4};

        const DEFAULT: &str = "default";

        #[derive(PartialEq)]
        enum DefaultFeatures {
            Enabled,
            Disabled,
        }

        #[derive(PartialEq)]
        enum DependencyState {
            Enabled,
            Disabled,
        }

        fn optional_dependency() -> PackageDependency {
            PackageDependency {
                name: DEP_NAME.into(),
                optional: true,
                ..PackageDependency::default()
            }
        }

        fn normal_dependency() -> PackageDependency {
            PackageDependency {
                name: DEP_NAME.into(),
                optional: false,
                ..PackageDependency::default()
            }
        }

        fn run_test(
            package_features_slice: &[(&str, &[&str])],
            enabled_features_slice: &[&str],
            default_features: DefaultFeatures,
            dependency: PackageDependency,
            dependency_state: DependencyState,
        ) {
            let mut test_package = default_cargo_package_with_features(
                package_features_slice,
                Some(enabled_features_slice),
            );
            test_package.default_features_disabled = default_features == DefaultFeatures::Disabled;

            let expected = dependency_state == DependencyState::Enabled;
            assert_eq!(expected, test_package.is_dependency_enabled(&dependency));
        }

        // not_optional_dependency
        #[test_case( &[(F1, &[])], &[], DefaultFeatures::Enabled, normal_dependency(), DependencyState::Enabled ; "not_optional_dependency")]
        // only default dependencies
        #[test_case(&[(DEFAULT, &[])], &[], DefaultFeatures::Enabled, optional_dependency(), DependencyState::Disabled ; "only_default_empty")]
        #[test_case(&[(DEFAULT, &[&format!("dep:{}", DEP_NAME)])], &[], DefaultFeatures::Disabled, optional_dependency(), DependencyState::Disabled ; "only_default_and_default_disabled")]
        #[test_case(&[(DEFAULT, &["for-sure-not-enabling"])], &[], DefaultFeatures::Enabled, optional_dependency(), DependencyState::Disabled ; "only_default_for_sure_not_enabling")]
        #[test_case(&[(DEFAULT, &[&format!("dep:{}", DEP_NAME)])], &[], DefaultFeatures::Enabled, optional_dependency(), DependencyState::Enabled ; "only_default_enabling")]
        // enabled by currently enabled features
        #[test_case(&[(F1, &[&format!("dep:{}", DEP_NAME)])], &[F1], DefaultFeatures::Enabled, optional_dependency(), DependencyState::Enabled ; "currently_enabled_one_feature")]
        #[test_case(&[(F1, &["for-sure-not-enabling"])], &[F1], DefaultFeatures::Enabled, optional_dependency(), DependencyState::Disabled ; "currently_enabled_for_sure_not_enabling")]
        #[test_case(&[(F1, &[&format!("dep:{}", DEP_NAME)]), (F2, &[F1]), (F3, &[F2])], &[F3], DefaultFeatures::Enabled, optional_dependency(), DependencyState::Enabled ; "currently_enabled_many_features_begin")]
        #[test_case(&[(DEFAULT, &[F1]), (F1, &[F2]), (F2, &[F3]), (F3, &[&format!("dep:{}", DEP_NAME)])], &[], DefaultFeatures::Enabled, optional_dependency(), DependencyState::Enabled ; "currently_enabled_many_features_end")]
        #[test_case(&[(DEFAULT, &[F1]), (F1, &[F2]), (F2, &[&format!("dep:{}", DEP_NAME)])], &[], DefaultFeatures::Disabled, optional_dependency(), DependencyState::Disabled ; "currently_enabled_many_features_end_default_disabled")]
        fn no_cycles(
            package_features_slice: &[(&str, &[&str])],
            enabled_features_slice: &[&str],
            default_features: DefaultFeatures,
            dependency: PackageDependency,
            dependency_state: DependencyState,
        ) {
            run_test(
                package_features_slice,
                enabled_features_slice,
                default_features,
                dependency,
                dependency_state,
            )
        }

        #[test_case(&[(F1, &[&format!("dep:{}", DEP_NAME)]), (F2, &[F4]), (F3, &[F4]), (F4, &[F2, F3])], &[F3, F4], DefaultFeatures::Enabled, optional_dependency(), DependencyState::Disabled ; "first" )]
        #[test_case(&[(F1, &[&format!("dep:{}", DEP_NAME)]), (F2, &[F3]), (F3, &[F2]), (F4, &[F1]), ], &[F2, F3, F4], DefaultFeatures::Enabled,optional_dependency(), DependencyState::Enabled ; "second" )]
        #[timeout(10000)]
        fn cycles(
            package_features_slice: &[(&str, &[&str])],
            enabled_features_slice: &[&str],
            default_features: DefaultFeatures,
            dependency: PackageDependency,
            dependency_state: DependencyState,
        ) {
            run_test(
                package_features_slice,
                enabled_features_slice,
                default_features,
                dependency,
                dependency_state,
            )
        }
    }
}