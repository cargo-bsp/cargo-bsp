use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::bsp_types::requests::{CreateCommand, Request};
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

    fn create_command(&self) -> Command {
        todo!()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TestParams {
    /** A sequence of build targets to test. */
    pub targets: Vec<BuildTargetIdentifier>,

    /** A unique identifier generated by the client to identify this request.
     * The server may include this id in triggered notifications or responses. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<String>,

    /** Optional arguments to the test execution engine. */
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<String>,

    /** Kind of data to expect in the `data` field. If this field is not set, the kind of data is not specified. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_kind: Option<String>,

    /** Language-specific metadata about for this test execution.
     * See ScalaTestParams as an example. */
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

    use crate::bsp_types::tests::{test_deserialization, test_serialization};

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

        let mut modified = test_data.clone();
        modified.origin_id = None;
        test_deserialization(
            r#"{"targets":[{"uri":""}],"arguments":["test_argument"],"dataKind":"test_dataKind","data":{"dataKey":"dataValue"}}"#,
            &modified,
        );
        modified = test_data.clone();
        modified.arguments = vec![];
        test_deserialization(
            r#"{"targets":[{"uri":""}],"originId":"test_originId","dataKind":"test_dataKind","data":{"dataKey":"dataValue"}}"#,
            &modified,
        );
        modified = test_data.clone();
        modified.data_kind = None;
        test_deserialization(
            r#"{"targets":[{"uri":""}],"originId":"test_originId","arguments":["test_argument"],"data":{"dataKey":"dataValue"}}"#,
            &modified,
        );
        modified.data = None;
        test_deserialization(
            r#"{"targets":[{"uri":""}],"originId":"test_originId","arguments":["test_argument"]}"#,
            &modified,
        );
    }

    #[test]
    fn test_result() {
        let test_data = TestResult {
            origin_id: Some("test_originId".to_string()),
            status_code: StatusCode::default(),
            data_kind: Some("test_dataKind".to_string()),
            data: Some(serde_json::json!({"dataKey": "dataValue"})),
        };

        test_serialization(
            &test_data,
            r#"{"originId":"test_originId","statusCode":2,"dataKind":"test_dataKind","data":{"dataKey":"dataValue"}}"#,
        );

        let mut modified = test_data.clone();
        modified.origin_id = None;
        test_serialization(
            &modified,
            r#"{"statusCode":2,"dataKind":"test_dataKind","data":{"dataKey":"dataValue"}}"#,
        );
        modified = test_data.clone();
        modified.data_kind = None;
        test_serialization(
            &modified,
            r#"{"originId":"test_originId","statusCode":2,"data":{"dataKey":"dataValue"}}"#,
        );
        modified.data = None;
        test_serialization(&modified, r#"{"originId":"test_originId","statusCode":2}"#);
    }
}
