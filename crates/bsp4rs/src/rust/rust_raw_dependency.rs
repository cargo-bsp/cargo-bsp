use serde::{Deserialize, Serialize};

use crate::*;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustRawDependency {
    /// The name of the dependency.
    pub name: String,
    /// Name to which this dependency is renamed when declared in Cargo.toml.
    /// This field allows to specify an alternative name for a dependency to use in a code,
    /// regardless of how it’s published (helpful for example if multiple dependencies
    /// have conflicting names).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rename: Option<String>,
    /// The dependency kind.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<RustDepKind>,
    /// The target platform for the dependency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Indicates whether this is an optional dependency.
    pub optional: bool,
    /// Indicates whether default features are enabled.
    pub uses_default_features: bool,
    /// A sequence of enabled features.
    pub features: BTreeSet<Feature>,
}

#[cfg(test)]
mod test {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn rust_raw_dependency() {
        let dependency = RustRawDependency {
            name: "test_name".to_string(),
            rename: Some("test_rename".to_string()),
            kind: Some(RustDepKind::NORMAL),
            target: Some("test_target".to_string()),
            optional: false,
            uses_default_features: false,
            features: BTreeSet::from(["test_feature".into()]),
        };

        assert_json_snapshot!(dependency, @r#"
        {
          "name": "test_name",
          "rename": "test_rename",
          "kind": "normal",
          "target": "test_target",
          "optional": false,
          "usesDefaultFeatures": false,
          "features": [
            "test_feature"
          ]
        }
        "#);

        assert_json_snapshot!(RustRawDependency::default(), @r#"
        {
          "name": "",
          "optional": false,
          "usesDefaultFeatures": false,
          "features": []
        }
        "#);
    }
}
