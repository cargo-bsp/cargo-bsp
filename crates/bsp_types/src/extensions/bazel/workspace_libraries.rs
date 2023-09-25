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

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceLibrariesResult {
    pub libraries: Vec<LibraryItem>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItem {
    pub id: BuildTargetIdentifier,
    pub dependencies: Vec<BuildTargetIdentifier>,
    pub jars: Vec<Jar>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Jar(pub String);

impl std::ops::Deref for Jar {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for Jar {
    fn from(input: String) -> Self {
        Self(input)
    }
}

impl From<&str> for Jar {
    fn from(input: &str) -> Self {
        Self(input.to_string())
    }
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
            jars: vec!["test_jar".into()],
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
