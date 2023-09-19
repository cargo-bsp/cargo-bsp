//! This file was created when WorkspaceLibraries request was not yet added to the BSP documentation
//! But Intellij-BSP client already requested it.

use serde::{Deserialize, Serialize};

use crate::requests::Request;
use crate::BuildTargetIdentifier;

// TODO: Add to protocol. Enum and structs not in smithy model, so can't be copied from bsp4rs

#[derive(Debug)]
pub enum WorkspaceLibraries {}

impl Request for WorkspaceLibraries {
    type Params = ();
    type Result = WorkspaceLibrariesResult;
    const METHOD: &'static str = "workspace/libraries";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct WorkspaceLibrariesResult {
    pub libraries: Vec<LibraryItem>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItem {
    pub id: BuildTargetIdentifier,
    pub dependencies: Vec<BuildTargetIdentifier>,
    pub jars: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn workspace_libraries_method() {
        assert_eq!(WorkspaceLibraries::METHOD, "workspace/libraries");
    }

    #[test]
    fn workspace_libraries_result() {
        let test_data = WorkspaceLibrariesResult {
            libraries: vec![LibraryItem::default()],
        };

        assert_json_snapshot!(test_data, @r#"
        {
          "libraries": [
            {
              "id": {
                "uri": ""
              },
              "dependencies": [],
              "jars": []
            }
          ]
        }
        "#);
    }

    #[test]
    fn library_item() {
        let test_data = LibraryItem {
            id: BuildTargetIdentifier::default(),
            dependencies: vec![BuildTargetIdentifier::default()],
            jars: vec!["test_jar".to_string()],
        };

        assert_json_snapshot!(test_data, @r#"
        {
          "id": {
            "uri": ""
          },
          "dependencies": [
            {
              "uri": ""
            }
          ],
          "jars": [
            "test_jar"
          ]
        }
        "#);
        assert_json_snapshot!(LibraryItem::default(), @r#"
        {
          "id": {
            "uri": ""
          },
          "dependencies": [],
          "jars": []
        }
        "#);
    }
}
