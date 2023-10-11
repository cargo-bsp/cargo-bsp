use serde::{Deserialize, Serialize};

use crate::*;
use std::collections::BTreeSet;

/// `RustTarget` contains data of the target as defined in Cargo metadata.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustTarget {
    /// The name of the target.
    pub name: String,
    /// Path to the root module of the crate.
    pub crate_root_url: URI,
    /// A target's kind.
    pub kind: RustTargetKind,
    /// Type of output that is produced by a crate during the build process.
    /// The crate type determines how the source code is compiled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crate_types: Option<Vec<RustCrateType>>,
    /// The Rust edition of the target.
    pub edition: RustEdition,
    /// Whether or not this target has doc tests enabled, and
    /// the target is compatible with doc testing.
    pub doctest: bool,
    /// A sequence of required features.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_features: Option<BTreeSet<Feature>>,
}

#[cfg(test)]
mod test {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn rust_target() {
        let target = RustTarget {
            name: "test_name".to_string(),
            crate_root_url: "test_crate_url".into(),
            kind: RustTargetKind::default(),
            crate_types: Some(vec![RustCrateType::default()]),
            edition: RustEdition::default(),
            doctest: false,
            required_features: Some(BTreeSet::from(["test_feature".into()])),
        };

        assert_json_snapshot!(target, @r#"
        {
          "name": "test_name",
          "crateRootUrl": "test_crate_url",
          "kind": 1,
          "crateTypes": [
            1
          ],
          "edition": "",
          "doctest": false,
          "requiredFeatures": [
            "test_feature"
          ]
        }
        "#);

        assert_json_snapshot!(RustTarget::default(), @r#"
        {
          "name": "",
          "crateRootUrl": "",
          "kind": 1,
          "edition": "",
          "doctest": false
        }
        "#);
    }
}
