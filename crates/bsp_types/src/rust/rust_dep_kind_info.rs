use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustDepKindInfo {
    /// The dependency kind.
    pub kind: RustDepKind,
    /// The target platform for the dependency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

#[cfg(test)]
mod test {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn rust_dep_kind_info() {
        let dep_kind_info = RustDepKindInfo {
            kind: RustDepKind::NORMAL,
            target: Some("test_target".to_string()),
        };

        assert_json_snapshot!(dep_kind_info, @r#"
        {
          "kind": "normal",
          "target": "test_target"
        }
        "#);

        assert_json_snapshot!(RustDepKindInfo::default(), @r#"
        {
          "kind": ""
        }
        "#);
    }
}
