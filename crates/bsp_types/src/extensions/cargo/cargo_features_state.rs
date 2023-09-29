use crate::extensions::{Feature, FeatureDependencyGraph};
use crate::BuildTargetIdentifier;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::requests::Request;

#[derive(Debug)]
pub enum CargoFeaturesState {}

/// The cargo features state request is sent from the client to the server to
/// query for the current state of the Cargo features. Provides also mapping
/// between Cargo packages and build target identifiers.
impl Request for CargoFeaturesState {
    type Params = ();
    type Result = CargoFeaturesStateResult;
    const METHOD: &'static str = "workspace/cargoFeaturesState";
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CargoFeaturesStateResult {
    /// The list of Cargo packages with assigned to them target
    /// identifiers and available features.
    pub packages_features: Vec<PackageFeatures>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageFeatures {
    /// The Cargo package identifier.
    pub package_id: String,
    /// The list of build target identifiers assigned to the Cargo package.
    pub targets: Vec<BuildTargetIdentifier>,
    /// The list of available features for the Cargo package.
    pub available_features: FeatureDependencyGraph,
    /// The list of enabled features for the Cargo package.
    pub enabled_features: BTreeSet<Feature>,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;
    use std::collections::BTreeMap;

    use super::*;

    const PACKAGE_ID: &str = "package_id";
    const PACKAGE_ID2: &str = "package_id2";
    const FEATURE: &str = "feature";
    const FEATURE2: &str = "feature2";
    const TARGET_ID: &str = "target";
    const TARGET_ID2: &str = "target2";

    fn example_package_features(pid: &str, f1: &str) -> PackageFeatures {
        PackageFeatures {
            package_id: pid.into(),
            enabled_features: vec![f1.into()].into_iter().collect(),
            available_features: BTreeMap::from([(f1.into(), BTreeSet::new())]).into(),
            targets: vec![
                BuildTargetIdentifier {
                    uri: TARGET_ID.into(),
                },
                BuildTargetIdentifier {
                    uri: TARGET_ID2.into(),
                },
            ],
        }
    }

    #[test]
    fn cargo_features_state_method() {
        assert_eq!(CargoFeaturesState::METHOD, "workspace/cargoFeaturesState");
    }

    #[test]
    fn package_features() {
        let test_data = example_package_features(PACKAGE_ID, FEATURE);

        assert_json_snapshot!(PackageFeatures::default(), @r#"
        {
          "packageId": "",
          "targets": [],
          "availableFeatures": {},
          "enabledFeatures": []
        }
        "#);
        assert_json_snapshot!(test_data, @r#"
        {
          "packageId": "package_id",
          "targets": [
            {
              "uri": "target"
            },
            {
              "uri": "target2"
            }
          ],
          "availableFeatures": {
            "feature": []
          },
          "enabledFeatures": [
            "feature"
          ]
        }
        "#);
    }

    #[test]
    fn cargo_features_state_result() {
        let test_data = CargoFeaturesStateResult {
            packages_features: vec![
                example_package_features(PACKAGE_ID, FEATURE),
                example_package_features(PACKAGE_ID2, FEATURE2),
            ],
        };

        assert_json_snapshot!(CargoFeaturesStateResult::default(), @r#"
        {
          "packagesFeatures": []
        }
        "#);
        assert_json_snapshot!(test_data, @r#"
        {
          "packagesFeatures": [
            {
              "packageId": "package_id",
              "targets": [
                {
                  "uri": "target"
                },
                {
                  "uri": "target2"
                }
              ],
              "availableFeatures": {
                "feature": []
              },
              "enabledFeatures": [
                "feature"
              ]
            },
            {
              "packageId": "package_id2",
              "targets": [
                {
                  "uri": "target"
                },
                {
                  "uri": "target2"
                }
              ],
              "availableFeatures": {
                "feature2": []
              },
              "enabledFeatures": [
                "feature2"
              ]
            }
          ]
        }
        "#);
    }
}
