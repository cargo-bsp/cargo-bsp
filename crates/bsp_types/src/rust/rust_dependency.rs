use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustDependency {
    /// The Package ID of the dependency.
    pub pkg: String,
    /// The name of the dependency's library target.
    /// If this is a renamed dependency, this is the new name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Array of dependency kinds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dep_kinds: Option<Vec<RustDepKindInfo>>,
}

#[cfg(test)]
mod test {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn rust_dependency() {
        let dependency = RustDependency {
            name: Some("test_name".to_string()),
            pkg: "test_target".to_string(),
            dep_kinds: Some(vec![RustDepKindInfo::default()]),
        };

        assert_json_snapshot!(dependency, @r#"
        {
          "pkg": "test_target",
          "name": "test_name",
          "depKinds": [
            {
              "kind": ""
            }
          ]
        }
        "#);

        assert_json_snapshot!(RustDependency::default(), @r#"
        {
          "pkg": ""
        }
        "#);
    }
}
