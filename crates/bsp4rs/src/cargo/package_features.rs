use serde::{Deserialize, Serialize};

use crate::*;
use std::collections::BTreeSet;

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
    use super::*;
    use crate::cargo::tests::{example_package_features, FEATURE, PACKAGE_ID};
    use insta::assert_json_snapshot;

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
}
