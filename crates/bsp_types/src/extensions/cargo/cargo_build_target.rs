use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::extensions::{Feature, RustEdition};

/// `CargoBuildTarget` is a basic data structure that contains
/// cargo-specific metadata.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CargoBuildTarget {
    pub edition: RustEdition,
    pub required_features: BTreeSet<Feature>,
}

#[cfg(test)]
mod tests {
    use crate::extensions::RustEdition;
    use insta::assert_json_snapshot;

    use super::*;

    #[test]
    fn cargo_build_target() {
        let test_data = CargoBuildTarget {
            edition: RustEdition::E2015,
            required_features: BTreeSet::from(["test_requiredFeature".into()]),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "edition": "2015",
          "requiredFeatures": [
            "test_requiredFeature"
          ]
        }
        "#
        );
        assert_json_snapshot!(CargoBuildTarget::default(),
            @r#"
        {
          "edition": "",
          "requiredFeatures": []
        }
        "#
        );
    }
}
