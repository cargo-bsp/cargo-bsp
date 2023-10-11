use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestResult {
    /// An optional request id to know the origin of this report.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<Identifier>,
    /// A status code for the execution.
    pub status_code: StatusCode,
    /// Language-specific metadata about the test result.
    /// See ScalaTestParams as an example.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub data: Option<TestResultData>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn test_result() {
        let test_data = TestResult {
            origin_id: Some("test_originId".into()),
            status_code: StatusCode::default(),
            data: Some(TestResultData::Other(OtherData {
                data_kind: "test_dataKind".to_string(),
                data: serde_json::json!({"dataKey": "dataValue"}),
            })),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "originId": "test_originId",
          "statusCode": 1,
          "dataKind": "test_dataKind",
          "data": {
            "dataKey": "dataValue"
          }
        }
        "#
        );
        assert_json_snapshot!(TestResult::default(),
            @r#"
        {
          "statusCode": 1
        }
        "#
        );
    }
}
