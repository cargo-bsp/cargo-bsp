use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::requests::Request;
use crate::{BuildTargetIdentifier, StatusCode};

#[derive(Debug)]
pub enum Run {}

impl Request for Run {
    type Params = RunParams;
    type Result = RunResult;
    const METHOD: &'static str = "buildTarget/run";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RunParams {
    /** The build target to run. */
    pub target: BuildTargetIdentifier,

    /** A unique identifier generated by the client to identify this request.
    The server may include this id in triggered notifications or responses. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<String>,

    /** Optional arguments to the executed application. */
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<String>,

    /** Kind of data to expect in the data field. If this field is not set, the kind of data is not specified. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_kind: Option<String>,

    /** Language-specific metadata for this execution.
    See ScalaMainClass as an example. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RunResult {
    /** An optional request id to know the origin of this report. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<String>,

    /** A status code for the execution. */
    pub status_code: StatusCode,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use crate::tests::test_deserialization;

    use super::*;

    #[test]
    fn run_method() {
        assert_eq!(Run::METHOD, "buildTarget/run");
    }

    #[test]
    fn run_params() {
        let test_data = RunParams {
            target: BuildTargetIdentifier::default(),
            origin_id: Some("test_originId".to_string()),
            arguments: vec!["test_argument".to_string()],
            data_kind: Some("test_dataKind".to_string()),
            data: Some(serde_json::json!({"dataKey": "dataValue"})),
        };

        test_deserialization(
            r#"{"target":{"uri":""},"originId":"test_originId","arguments":["test_argument"],"dataKind":"test_dataKind","data":{"dataKey":"dataValue"}}"#,
            &test_data,
        );

        test_deserialization(r#"{"target":{"uri":""}}"#, &RunParams::default());
    }

    #[test]
    fn run_result() {
        let test_data = RunResult {
            origin_id: Some("test_originId".to_string()),
            status_code: StatusCode::default(),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "originId": "test_originId",
          "statusCode": 2
        }
        "#
        );
        assert_json_snapshot!(RunResult::default(),
            @r#"
        {
          "statusCode": 2
        }
        "#
        );
    }
}
