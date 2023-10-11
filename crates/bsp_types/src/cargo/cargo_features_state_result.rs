use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CargoFeaturesStateResult {
    /// The list of Cargo packages with assigned to them target
    /// identifiers and available features.
    pub packages_features: Vec<PackageFeatures>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cargo::tests::{
        example_package_features, FEATURE, FEATURE2, PACKAGE_ID, PACKAGE_ID2,
    };
    use insta::assert_json_snapshot;

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
