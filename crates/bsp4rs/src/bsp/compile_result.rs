use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileResult {
    /// An optional request id to know the origin of this report.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<Identifier>,
    /// A status code for the execution.
    pub status_code: StatusCode,
    /// A field containing language-specific information, like products
    /// of compilation or compiler-specific metadata the client needs to know.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub data: Option<CompileResultData>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn compile_result() {
        let test_data = CompileResult {
            origin_id: Some("test_message".into()),
            status_code: StatusCode::default(),
            data: Some(CompileResultData::Other(OtherData {
                data_kind: "test_dataKind".to_string(),
                data: serde_json::json!({"dataKey": "dataValue"}),
            })),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "originId": "test_message",
          "statusCode": 1,
          "dataKind": "test_dataKind",
          "data": {
            "dataKey": "dataValue"
          }
        }
        "#
        );
        assert_json_snapshot!(CompileResult::default(),
            @r#"
        {
          "statusCode": 1
        }
        "#
        );
    }
}
