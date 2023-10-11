use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceLibrariesResult {
    pub libraries: Vec<LibraryItem>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

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
              "jars": [],
              "sourceJars": []
            }
          ]
        }
        "#);
    }
}
