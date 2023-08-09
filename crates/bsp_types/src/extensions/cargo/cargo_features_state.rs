use super::Feature;
use crate::BuildTargetIdentifier;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::requests::{FeaturesDependencyGraph, Request};

#[derive(Debug)]
pub enum CargoFeaturesState {}

impl Request for CargoFeaturesState {
    type Params = ();
    type Result = CargoFeaturesStateResult;
    const METHOD: &'static str = "workspace/cargoFeaturesState";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CargoFeaturesStateResult {
    pub packages_features: Vec<PackageFeatures>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PackageFeatures {
    pub package_id: String,
    pub targets: Vec<BuildTargetIdentifier>,
    pub enabled_features: BTreeSet<Feature>,
    pub available_features: FeaturesDependencyGraph,
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
        let mut available_features = BTreeMap::new();
        available_features.insert(f1.into(), vec![]);
        PackageFeatures {
            package_id: pid.into(),
            enabled_features: vec![f1.into()].into_iter().collect(),
            available_features,
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

        assert_json_snapshot!(PackageFeatures::default(), @r###"
        {
          "packageId": "",
          "targets": [],
          "enabledFeatures": [],
          "availableFeatures": {}
        }
        "###);
        assert_json_snapshot!(test_data, @r###"
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
          "enabledFeatures": [
            "feature"
          ],
          "availableFeatures": {
            "feature": []
          }
        }
        "###);
    }

    #[test]
    fn cargo_features_state_result() {
        let test_data = CargoFeaturesStateResult {
            packages_features: vec![
                example_package_features(PACKAGE_ID, FEATURE),
                example_package_features(PACKAGE_ID2, FEATURE2),
            ],
        };

        assert_json_snapshot!(CargoFeaturesStateResult::default(), @r###"
        {
          "packagesFeatures": []
        }
        "###);
        assert_json_snapshot!(test_data, @r###"
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
              "enabledFeatures": [
                "feature"
              ],
              "availableFeatures": {
                "feature": []
              }
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
              "enabledFeatures": [
                "feature2"
              ],
              "availableFeatures": {
                "feature2": []
              }
            }
          ]
        }
        "###);
    }
}
