use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceBuildTargetsResult {
    /// The build targets in this workspace that
    /// contain sources with the given language ids.
    pub targets: Vec<BuildTarget>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn workspace_build_targets_result() {
        let test_data = WorkspaceBuildTargetsResult {
            targets: vec![BuildTarget::default()],
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "targets": [
            {
              "id": {
                "uri": ""
              },
              "tags": [],
              "languageIds": [],
              "dependencies": [],
              "capabilities": {}
            }
          ]
        }
        "#
        );
        assert_json_snapshot!(WorkspaceBuildTargetsResult::default(),
            @r#"
        {
          "targets": []
        }
        "#
        );
    }
}
