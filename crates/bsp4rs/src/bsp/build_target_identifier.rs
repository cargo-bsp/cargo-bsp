use serde::{Deserialize, Serialize};

use crate::*;

/// A unique identifier for a target, can use any URI-compatible encoding as long as it is unique within the workspace.
/// Clients should not infer metadata out of the URI structure such as the path or query parameters, use `BuildTarget` instead.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildTargetIdentifier {
    /// The target's Uri
    pub uri: URI,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn build_target_identifier() {
        let test_data = BuildTargetIdentifier {
            uri: "test_uri".into(),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "uri": "test_uri"
        }
        "#
        );
        assert_json_snapshot!(BuildTargetIdentifier::default(),
            @r#"
        {
          "uri": ""
        }
        "#
        );
    }
}
