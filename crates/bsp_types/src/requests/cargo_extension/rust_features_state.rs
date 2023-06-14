use super::Feature;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::requests::Request;

#[derive(Debug)]
pub enum RustFeaturesState {}

impl Request for RustFeaturesState {
    type Params = RustFeaturesStateParams;
    type Result = RustFeaturesStateResult;
    const METHOD: &'static str = "buildTarget/rustFeaturesState";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustFeaturesStateParams {}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustFeaturesStateResult {
    pub packages_features: Vec<PackageFeatures>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PackageFeatures {
    pub package_id: String,
    pub enabled_features: BTreeSet<Feature>,
    pub available_features: BTreeMap<Feature, Vec<Feature>>,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use crate::tests::test_deserialization;

    use super::*;

    const PACKAGE_ID: &str = "package_id";
    const PACKAGE_ID2: &str = "package_id2";
    const FEATURE: &str = "example_feature";
    const FEATURE2: &str = "example_feature2";
    const FEATURE3: &str = "example_feature3";

    fn example_package_features(pid: &str, f1: &str, f2: &str, f3: &str) -> PackageFeatures {
        PackageFeatures {
            package_id: pid.into(),
            enabled_features: vec![f1.into()].into_iter().collect(),
            available_features: vec![
                (f1.into(), vec![f2.into(), f3.into()]),
                (f3.into(), vec![f1.into()]),
            ]
            .into_iter()
            .collect(),
        }
    }

    #[test]
    fn rust_features_state_method() {
        assert_eq!(RustFeaturesState::METHOD, "buildTarget/rustFeaturesState");
    }

    #[test]
    fn rust_features_state_params() {
        test_deserialization(r#"{}"#, &RustFeaturesStateParams {});
    }

    #[test]
    fn package_features() {
        let test_data = example_package_features(PACKAGE_ID, FEATURE, FEATURE2, FEATURE3);

        assert_json_snapshot!(PackageFeatures::default(), @r###"
        {
          "packageId": "",
          "enabledFeatures": [],
          "availableFeatures": {}
        }
        "###);
        assert_json_snapshot!(test_data, @r###"
        {
          "packageId": "package_id",
          "enabledFeatures": [
            "example_feature"
          ],
          "availableFeatures": {
            "example_feature": [
              "example_feature2",
              "example_feature3"
            ],
            "example_feature3": [
              "example_feature"
            ]
          }
        }
        "###);
    }

    #[test]
    fn rust_features_state_result() {
        let test_data = RustFeaturesStateResult {
            packages_features: vec![
                example_package_features(PACKAGE_ID, FEATURE, FEATURE2, FEATURE3),
                example_package_features(PACKAGE_ID2, FEATURE3, FEATURE2, FEATURE),
            ],
        };

        assert_json_snapshot!(RustFeaturesStateResult::default(), @r###"
        {
          "packagesFeatures": []
        }
        "###);
        assert_json_snapshot!(test_data, @r###"
        {
          "packagesFeatures": [
            {
              "packageId": "package_id",
              "enabledFeatures": [
                "example_feature"
              ],
              "availableFeatures": {
                "example_feature": [
                  "example_feature2",
                  "example_feature3"
                ],
                "example_feature3": [
                  "example_feature"
                ]
              }
            },
            {
              "packageId": "package_id2",
              "enabledFeatures": [
                "example_feature3"
              ],
              "availableFeatures": {
                "example_feature": [
                  "example_feature3"
                ],
                "example_feature3": [
                  "example_feature2",
                  "example_feature"
                ]
              }
            }
          ]
        }
        "###);
    }
}
