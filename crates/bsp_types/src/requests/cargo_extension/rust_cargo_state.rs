use super::Feature;
use crate::BuildTargetIdentifier;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::requests::Request;

#[derive(Debug)]
pub enum CargoFeaturesState {}

impl Request for CargoFeaturesState {
    type Params = CargoFeaturesStateParams;
    type Result = CargoFeaturesStateResult;
    const METHOD: &'static str = "buildTarget/CargoFeaturesState";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CargoFeaturesStateParams {}

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
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use crate::tests::test_deserialization;

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
        assert_eq!(CargoFeaturesState::METHOD, "buildTarget/CargoFeaturesState");
    }

    #[test]
    fn cargo_features_state_params() {
        test_deserialization(r#"{}"#, &CargoFeaturesStateParams {});
    }

    #[test]
    fn package_features() {
        let test_data = example_package_features(PACKAGE_ID, FEATURE);

        assert_json_snapshot!(PackageFeatures::default(), @r###"
        {
          "packageId": "",
          "targets": [],
          "enabledFeatures": []
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
          ]
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
              "enabledFeatures": [
                "feature2"
              ]
            }
          ]
        }
        "###);
    }
}
