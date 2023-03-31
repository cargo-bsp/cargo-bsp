use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::bsp_types::requests::{CreateCommand, Request};
use crate::bsp_types::{BuildTargetIdentifier, StatusCode};

/*
NOTE THAT:
Response has error field defined as follows:
error: JSON-RPC code and message set in case an exception happens during the request.
*/

#[derive(Debug)]
pub struct Compile {}

impl Request for Compile {
    type Params = CompileParams;
    type Result = CompileResult;
    const METHOD: &'static str = "buildTarget/compile";
}

impl CreateCommand for CompileParams {
    fn origin_id(&self) -> Option<String> {
        self.origin_id.clone()
    }

    fn create_command(&self) -> Command {
        todo!()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompileParams {
    /** A sequence of build targets to compile. */
    pub targets: Vec<BuildTargetIdentifier>,

    /** A unique identifier generated by the client to identify this request.
     * The server may include this id in triggered notifications or responses. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<String>,

    /** Optional arguments to the compilation process. */
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompileResult {
    /** An optional request id to know the origin of this report. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<String>,

    /** A status code for the execution. */
    pub status_code: StatusCode,

    /** Kind of data to expect in the `data` field. If this field is not set, the kind of data is not specified. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_kind: Option<String>,

    /** A field containing language-specific information, like products
     * of compilation or compiler-specific metadata the client needs to know. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use crate::bsp_types::tests::test_deserialization;

    use super::*;

    #[test]
    fn compile_method() {
        assert_eq!(Compile::METHOD, "buildTarget/compile");
    }

    #[test]
    fn compile_params() {
        let test_data = CompileParams {
            targets: vec![BuildTargetIdentifier::default()],
            origin_id: Some("test_message".to_string()),
            arguments: vec!["test_argument".to_string()],
        };

        test_deserialization(
            r#"{"targets":[{"uri":""}],"originId":"test_message","arguments":["test_argument"]}"#,
            &test_data,
        );

        test_deserialization(r#"{"targets":[]}"#, &CompileParams::default());
    }

    #[test]
    fn compile_result() {
        let test_data = CompileResult {
            origin_id: Some("test_message".to_string()),
            status_code: StatusCode::default(),
            data_kind: Some("test_data_kind".to_string()),
            data: Some(serde_json::json!({"dataKey": "dataValue"})),
        };

        assert_json_snapshot!(test_data,
            @r###"
        {
          "originId": "test_message",
          "statusCode": 2,
          "dataKind": "test_data_kind",
          "data": {
            "dataKey": "dataValue"
          }
        }
        "###
        );
        assert_json_snapshot!(CompileResult::default(),
            @r###"
        {
          "statusCode": 2
        }
        "###
        );
    }
}
