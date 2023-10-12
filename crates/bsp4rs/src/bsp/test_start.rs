use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestStart {
    /// Name or description of the test.
    pub display_name: String,
    /// Source location of the test, as LSP location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn test_start() {
        let test_data = TestStart {
            display_name: "test_name".to_string(),
            location: Some(Location {
                uri: "file:///test".into(),
                range: Range::default(),
            }),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "displayName": "test_name",
          "location": {
            "uri": "file:///test",
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            }
          }
        }
        "#
        );
        assert_json_snapshot!(TestStart::default(),
            @r#"
        {
          "displayName": ""
        }
        "#
        );
    }
}
