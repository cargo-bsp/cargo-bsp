use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItem {
    pub id: BuildTargetIdentifier,
    pub dependencies: Vec<BuildTargetIdentifier>,
    pub jars: Vec<Jar>,
    pub source_jars: Vec<Jar>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn library_item() {
        let test_data = LibraryItem {
            id: BuildTargetIdentifier::default(),
            dependencies: vec![BuildTargetIdentifier::default()],
            jars: vec!["test_jar".into()],
            source_jars: vec!["test_jar".into()],
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
          ],
          "sourceJars": [
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
          "jars": [],
          "sourceJars": []
        }
        "#);
    }
}
