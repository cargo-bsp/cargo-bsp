use crate::bsp_types::mappings::build_target::bsp_build_target_from_cargo_target;
use crate::bsp_types::{BuildTarget, BuildTargetIdentifier};
use crate::project_model::package_dependency::PackageDependency;
use cargo_metadata::camino::Utf8PathBuf;
use log::{error, warn};
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;

pub type Feature = String;

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
    pub enabled_features: Vec<Feature>,

    /// If true, default features are disabled. Does not apply when default features
    /// are not defined in package's manifest
    pub default_features_disabled: bool,

    /// Hashmap where key is a feature name and the value are names of other features it enables.
    /// Includes pair for default features if default is defined
    pub package_features: HashMap<Feature, Vec<Feature>>,
}

impl CargoPackage {
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
    fn check_if_feature_enables_dependency(feature: &str, dependency_name: &String) -> bool {
        feature == format!("dep:{}", dependency_name)
            || feature.starts_with(&format!("{}/", dependency_name))
    }

    /// Checks if a feature was defined in the `Cargo.toml`. Used to skip features that have the form:
    /// "dep:package_name" or "package_name/feature_name" or "package_name?/feature_name" as they
    /// are not included in the cargo metadata features Hashmap
    fn is_defined_feature(&self, feature: &str) -> bool {
        self.package_features.contains_key(feature)
    }

    /// Checks whether a dependency is enabled by the current set of enabled features.
    /// Runs BFS on the features graph starting from default (if defined and not disabled)
    /// and the enabled features
    fn is_dependency_enabled(&self, dependency: &PackageDependency) -> bool {
        if !dependency.optional {
            return true;
        }

        let mut next_features: VecDeque<Feature> = VecDeque::from(self.enabled_features.clone());
        if !self.default_features_disabled && self.is_defined_feature("default") {
            next_features.push_back(Feature::from("default"));
        }

        let mut checked_features: HashSet<Feature> =
            HashSet::from_iter(next_features.iter().cloned());

        while let Some(f) = next_features.pop_front() {
            let dependent_features = self.package_features.get(&f);
            if dependent_features.is_none() {
                error!("Feature {} not found in package {}", f, self.name);
                continue;
            }

            for df in dependent_features.unwrap() {
                if CargoPackage::check_if_feature_enables_dependency(df, &dependency.name) {
                    return true;
                }
                if checked_features.contains(df) || !self.is_defined_feature(df) {
                    continue;
                }
                checked_features.insert(df.clone());
                next_features.push_back(df.clone());
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
            if self.package_features.get(f).is_none() || self.enabled_features.contains(f) {
                warn!(
                    "Can't enable feature {}. It doesn't exist or is already enabled.",
                    f
                );
                return;
            }
            self.enabled_features.push(f.clone())
        });
    }

    /// Disables a list of features if they exist and are enabled
    pub fn disable_features(&mut self, features: &[Feature]) {
        features.iter().for_each(|f| {
            if self.package_features.get(f).is_none() || !self.enabled_features.contains(f) {
                warn!(
                    "Can't disable feature {}. It doesn't exist or isn't enabled.",
                    f
                );
                return;
            }
            self.enabled_features.retain(|x| x != f);
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
    use crate::project_model::package::{CargoPackage, Feature};
    use std::collections::HashMap;

    const DEP_NAME: &str = "dependency-name";
    const F1: &str = "feature1";
    const F2: &str = "feature2";
    const F3: &str = "feature3";
    const F4: &str = "feature4";

    fn create_package_features(slice_map: &[(&str, &[&str])]) -> HashMap<Feature, Vec<Feature>> {
        let mut feature_map: HashMap<Feature, Vec<Feature>> = HashMap::new();
        slice_map.iter().for_each(|(k, v)| {
            feature_map.insert(k.to_string(), v.iter().map(|s| s.to_string()).collect());
        });
        feature_map
    }

    fn create_feature_vector_from_slices(slices: &[&str]) -> Vec<Feature> {
        slices.iter().map(|f| f.to_string()).collect()
    }

    #[test]
    fn test_check_if_enabling_feature() {
        struct TestCase {
            feature: Feature,
            expected_with_dep_name_in_feature: bool,
            expected_without_dep_name_in_feature: bool,
        }

        impl TestCase {
            fn new(feature: Feature, expected_with_dep_name_in_feature: bool) -> Self {
                Self {
                    feature,
                    expected_with_dep_name_in_feature,
                    expected_without_dep_name_in_feature: false,
                }
            }
        }

        let test_cases = vec![
            TestCase::new("feature-name".into(), false),
            TestCase::new(DEP_NAME.into(), false),
            TestCase::new(format!("dep:{}", DEP_NAME), true),
            TestCase::new(format!("{}/feature", DEP_NAME), true),
            TestCase::new(format!("{}?/feature", DEP_NAME), false),
        ];
        test_cases.iter().for_each(|tc| {
            assert_eq!(
                tc.expected_with_dep_name_in_feature,
                CargoPackage::check_if_feature_enables_dependency(
                    &tc.feature,
                    &String::from(DEP_NAME)
                )
            );
            assert_eq!(
                tc.expected_without_dep_name_in_feature,
                CargoPackage::check_if_feature_enables_dependency(
                    &tc.feature,
                    &String::from("other-dependency-name")
                )
            );
        });
    }

    #[test]
    fn test_is_defined_feature() {
        struct TestCase {
            test_package: CargoPackage,
            feature: Feature,
            expected: bool,
        }
        impl TestCase {
            fn new(
                package_features_slice: &[(&str, &[&str])],
                feature: &str,
                expected: bool,
            ) -> Self {
                let test_package = CargoPackage {
                    package_features: create_package_features(package_features_slice),
                    ..CargoPackage::default()
                };
                Self {
                    test_package,
                    feature: feature.into(),
                    expected,
                }
            }
        }
        let test_cases = vec![
            TestCase::new(&[(F1, &[])], F1, true),
            TestCase::new(&[(F1, &[])], F2, false),
            TestCase::new(&[(F1, &[F2])], F2, false),
            TestCase::new(
                &[(F1, &[F2]), (F2, &[F3]), (F3, &["dep:name"])],
                "dep:name",
                false,
            ),
            TestCase::new(&[(F1, &[]), (F2, &[]), (F3, &[])], F3, true),
        ];

        test_cases.iter().for_each(|tc| {
            assert_eq!(tc.expected, tc.test_package.is_defined_feature(&tc.feature));
        });
    }

    mod test_is_dependency_enabled {
        use super::{
            create_feature_vector_from_slices, create_package_features, DEP_NAME, F1, F2, F3, F4,
        };
        use crate::project_model::package::CargoPackage;
        use crate::project_model::package_dependency::PackageDependency;
        use ntest::timeout;

        const DEFAULT: &str = "default";

        struct TestCase {
            test_package: CargoPackage,
            dependency: PackageDependency,
            expected: bool,
        }

        impl TestCase {
            fn new(
                package_features_slice: &[(&str, &[&str])],
                enabled_features_slice: &[&str],
                default_features: DefaultFeatures,
                dependency: PackageDependency,
                dependency_state: DependencyState,
            ) -> Self {
                let test_package = CargoPackage {
                    package_features: create_package_features(package_features_slice),
                    enabled_features: create_feature_vector_from_slices(enabled_features_slice),
                    default_features_disabled: default_features == DefaultFeatures::Disabled,
                    ..CargoPackage::default()
                };
                Self {
                    test_package,
                    dependency,
                    expected: dependency_state == DependencyState::Enabled,
                }
            }
        }

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

        fn run_tests(test_cases: &[TestCase]) {
            test_cases.iter().for_each(|tc| {
                assert_eq!(
                    tc.expected,
                    tc.test_package.is_dependency_enabled(&tc.dependency)
                );
            });
        }

        #[test]
        fn test_not_optional_dependency() {
            run_tests(&[TestCase::new(
                &[(F1, &[])],
                &[],
                DefaultFeatures::Enabled,
                PackageDependency {
                    optional: false,
                    ..PackageDependency::default()
                },
                DependencyState::Enabled,
            )]);
        }

        #[test]
        fn test_only_default_dependencies() {
            let test_cases = vec![
                TestCase::new(
                    &[(DEFAULT, &[])],
                    &[],
                    DefaultFeatures::Enabled,
                    optional_dependency(),
                    DependencyState::Disabled,
                ),
                TestCase::new(
                    &[(DEFAULT, &[&format!("dep:{}", DEP_NAME)])],
                    &[],
                    DefaultFeatures::Disabled,
                    optional_dependency(),
                    DependencyState::Disabled,
                ),
                TestCase::new(
                    &[(DEFAULT, &["for-sure-not-enabling"])],
                    &[],
                    DefaultFeatures::Enabled,
                    optional_dependency(),
                    DependencyState::Disabled,
                ),
                TestCase::new(
                    &[(DEFAULT, &[&format!("dep:{}", DEP_NAME)])],
                    &[],
                    DefaultFeatures::Enabled,
                    optional_dependency(),
                    DependencyState::Enabled,
                ),
            ];

            run_tests(&test_cases);
        }

        #[test]
        fn enabled_by_currently_enabled_features() {
            let test_cases = [
                TestCase::new(
                    &[(F1, &[&format!("dep:{}", DEP_NAME)])],
                    &[F1],
                    DefaultFeatures::Enabled,
                    optional_dependency(),
                    DependencyState::Enabled,
                ),
                TestCase::new(
                    &[(F1, &["for-sure-not-enabling"])],
                    &[F1],
                    DefaultFeatures::Enabled,
                    optional_dependency(),
                    DependencyState::Disabled,
                ),
                TestCase::new(
                    &[
                        (F1, &[&format!("dep:{}", DEP_NAME)]),
                        (F2, &[F1]),
                        (F3, &[F2]),
                    ],
                    &[F3],
                    DefaultFeatures::Enabled,
                    optional_dependency(),
                    DependencyState::Enabled,
                ),
                TestCase::new(
                    &[
                        (DEFAULT, &[F1]),
                        (F1, &[F2]),
                        (F2, &[F3]),
                        (F3, &[&format!("dep:{}", DEP_NAME)]),
                    ],
                    &[],
                    DefaultFeatures::Enabled,
                    optional_dependency(),
                    DependencyState::Enabled,
                ),
                TestCase::new(
                    &[
                        (DEFAULT, &[F1]),
                        (F1, &[F2]),
                        (F2, &[&format!("dep:{}", DEP_NAME)]),
                    ],
                    &[],
                    DefaultFeatures::Disabled,
                    optional_dependency(),
                    DependencyState::Disabled,
                ),
            ];

            run_tests(&test_cases);
        }

        #[test]
        #[timeout(10000)]
        fn cycled_features() {
            let test_cases = [
                TestCase::new(
                    &[
                        (F1, &[&format!("dep:{}", DEP_NAME)]),
                        (F2, &[F4]),
                        (F3, &[F4]),
                        (F4, &[F2, F3]),
                    ],
                    &[F3, F4],
                    DefaultFeatures::Enabled,
                    optional_dependency(),
                    DependencyState::Disabled,
                ),
                TestCase::new(
                    &[
                        (F1, &[&format!("dep:{}", DEP_NAME)]),
                        (F2, &[F3]),
                        (F3, &[F2]),
                        (F4, &[F1]),
                    ],
                    &[F2, F3, F4],
                    DefaultFeatures::Enabled,
                    optional_dependency(),
                    DependencyState::Enabled,
                ),
            ];

            run_tests(&test_cases);
        }
    }

    mod test_enabling_and_disabling_features {
        use super::{create_feature_vector_from_slices, F1, F2, F3};
        use crate::project_model::package::tests::create_package_features;
        use crate::project_model::package::{CargoPackage, Feature};

        struct TestCase {
            features_to_toggle: Vec<Feature>,
            test_package: CargoPackage,
            expected: Vec<Feature>,
        }

        impl TestCase {
            fn new(
                defined_features: &[&str],
                features_to_toggle: &[&str],
                enabled_features_slice: &[&str],
                expected: &[&str],
            ) -> Self {
                let package_features = create_package_features(
                    &defined_features
                        .iter()
                        .map(|&f| (f, &[] as &[&str]))
                        .collect::<Vec<(&str, &[&str])>>(),
                );
                Self {
                    features_to_toggle: create_feature_vector_from_slices(features_to_toggle),
                    test_package: CargoPackage {
                        enabled_features: create_feature_vector_from_slices(enabled_features_slice),
                        package_features,
                        ..CargoPackage::default()
                    },
                    expected: create_feature_vector_from_slices(expected),
                }
            }
        }

        #[test]
        fn test_enabling_features() {
            let test_cases = vec![
                TestCase::new(&[], &[], &[], &[]),
                TestCase::new(&[], &[], &[F1], &[F1]),
                TestCase::new(&[], &[F2], &[], &[]),
                TestCase::new(&[F1], &[F2], &[F1], &[F1]),
                TestCase::new(&[F1], &[F1], &[], &[F1]),
                TestCase::new(&[F1], &[F1], &[F1], &[F1]),
                TestCase::new(&[F1, F2], &[F2], &[F1], &[F1, F2]),
                TestCase::new(&[F1, F2], &[F1, F2], &[F1], &[F1, F2]),
            ];

            test_cases.into_iter().for_each(|mut tc| {
                tc.test_package.enable_features(&tc.features_to_toggle);
                assert_eq!(tc.expected, tc.test_package.enabled_features);
            });
        }

        #[test]
        fn test_disabling_features() {
            let test_cases = vec![
                TestCase::new(&[], &[], &[], &[]),
                TestCase::new(&[], &[F1], &[], &[]),
                TestCase::new(&[F1], &[F1], &[F1], &[]),
                TestCase::new(&[F1], &[F2], &[F1], &[F1]),
                TestCase::new(&[F1, F2], &[F2, F3], &[F1], &[F1]),
                TestCase::new(&[F1, F2, F3], &[F2, F3], &[F1, F2, F3], &[F1]),
                TestCase::new(&[F1, F2, F3], &[F1, F2, F3], &[F1, F2, F3], &[]),
            ];

            test_cases.into_iter().for_each(|mut tc| {
                tc.test_package.disable_features(&tc.features_to_toggle);
                assert_eq!(tc.expected, tc.test_package.enabled_features);
            });
        }
    }
}
