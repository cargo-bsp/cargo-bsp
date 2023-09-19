use serde::{Deserialize, Serialize};

use crate::requests::Request;
use crate::BuildTarget;

#[derive(Debug)]
pub enum WorkspaceBuildTargets {}

/// The workspace build targets request is sent from the client to the server to ask
/// for the list of all available build targets in the workspace.
impl Request for WorkspaceBuildTargets {
    type Params = ();
    type Result = WorkspaceBuildTargetsResult;
    const METHOD: &'static str = "workspace/buildTargets";
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceBuildTargetsResult {
    /// The build targets in this workspace that
    /// contain sources with the given language ids.
    pub targets: Vec<BuildTarget>,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use super::*;

    #[test]
    fn workspace_build_targets_method() {
        assert_eq!(WorkspaceBuildTargets::METHOD, "workspace/buildTargets");
    }

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
