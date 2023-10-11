use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestFinish {
    /// Name or description of the test.
    pub display_name: String,
    /// Information about completion of the test, for example an error message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Completion status of the test.
    pub status: TestStatus,
    /// Source location of the test, as LSP location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    /// Optionally, structured metadata about the test completion.
    /// For example: stack traces, expected/actual values.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub data: Option<TestFinishData>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn test_finish() {
        let test_data = TestFinish {
            display_name: "test_name".to_string(),
            message: Some("test_message".to_string()),
            status: TestStatus::default(),
            location: Some(Location {
                uri: "file:///test".into(),
                range: Range::default(),
            }),
            data: Some(TestFinishData::Other(OtherData {
                data_kind: "test_dataKind".to_string(),
                data: serde_json::json!({"dataKey": "dataValue"}),
            })),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "displayName": "test_name",
          "message": "test_message",
          "status": 1,
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
          },
          "dataKind": "test_dataKind",
          "data": {
            "dataKey": "dataValue"
          }
        }
        "#
        );
        assert_json_snapshot!(TestFinish::default(),
            @r#"
        {
          "displayName": "",
          "status": 1
        }
        "#
        );
    }
}
