use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::bsp_types::requests::{CreateCommand, CreateResult, Request};
use crate::bsp_types::{BuildTargetIdentifier, StatusCode};

#[derive(Debug)]
pub enum Test {}

impl Request for Test {
    type Params = TestParams;
    type Result = TestResult;
    const METHOD: &'static str = "buildTarget/test";
}

impl CreateCommand for TestParams {
    fn origin_id(&self) -> Option<String> {
        self.origin_id.clone()
    }

    fn create_command(&self, root: PathBuf) -> Command {
        let mut cmd = Command::new(toolchain::cargo());
        cmd.current_dir(root)
            .args([
                "test",
                "--message-format=json",
                "--",
                "--show-output",
                "-Z",
                "unstable-options",
                "--format=json",
            ])
            .args(self.arguments.clone());
        cmd
    }
}

impl CreateResult<TestResult> for TestParams {
    fn create_result(&self, status_code: StatusCode) -> TestResult {
        TestResult {
            origin_id: self.origin_id.clone(),
            status_code,
            data_kind: None,
            data: None,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TestParams {
    /** A sequence of build targets to test. */
    pub targets: Vec<BuildTargetIdentifier>,

    /** A unique identifier generated by the client to identify this request.
    The server may include this id in triggered notifications or responses. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<String>,

    /** Optional arguments to the test execution engine. */
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<String>,

    /** Kind of data to expect in the `data` field. If this field is not set, the kind of data is not specified. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_kind: Option<String>,

    /** Language-specific metadata about for this test execution.
    See ScalaTestParams as an example. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TestResult {
    /** An optional request id to know the origin of this report. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<String>,

    /** A status code for the execution. */
    pub status_code: StatusCode,

    /** Kind of data to expect in the `data` field. If this field is not set, the kind of data is not specified. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_kind: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[cfg(test)]
mod tests {
    use std::vec;

    use insta::assert_json_snapshot;

    use crate::bsp_types::tests::test_deserialization;

    use super::*;

    #[test]
    fn test_method() {
        assert_eq!(Test::METHOD, "buildTarget/test");
    }

    #[test]
    fn test_params() {
        let test_data = TestParams {
            targets: vec![BuildTargetIdentifier::default()],
            origin_id: Some("test_originId".to_string()),
            arguments: vec!["test_argument".to_string()],
            data_kind: Some("test_dataKind".to_string()),
            data: Some(serde_json::json!({"dataKey": "dataValue"})),
        };

        test_deserialization(
            r#"{"targets":[{"uri":""}],"originId":"test_originId","arguments":["test_argument"],"dataKind":"test_dataKind","data":{"dataKey":"dataValue"}}"#,
            &test_data,
        );

        test_deserialization(r#"{"targets":[]}"#, &TestParams::default());
    }

    #[test]
    fn test_result() {
        let test_data = TestResult {
            origin_id: Some("test_originId".to_string()),
            status_code: StatusCode::default(),
            data_kind: Some("test_dataKind".to_string()),
            data: Some(serde_json::json!({"dataKey": "dataValue"})),
        };

        assert_json_snapshot!(test_data,
            @r###"
        {
          "originId": "test_originId",
          "statusCode": 2,
          "dataKind": "test_dataKind",
          "data": {
            "dataKey": "dataValue"
          }
        }
        "###
        );
        assert_json_snapshot!(TestResult::default(),
            @r###"
        {
          "statusCode": 2
        }
        "###
        );
    }
}
