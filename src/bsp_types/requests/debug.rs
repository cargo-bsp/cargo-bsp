use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::bsp_types::requests::Request;
use crate::bsp_types::{BuildTargetIdentifier, Uri};

#[derive(Debug)]
pub enum DebugSession {}

impl Request for DebugSession {
    type Params = DebugSessionParams;
    type Result = DebugSessionAddress;
    const METHOD: &'static str = "debugSession/start";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DebugSessionParams {
    /** A sequence of build targets affected by the debugging action. */
    pub targets: Vec<BuildTargetIdentifier>,

    /** The kind of data to expect in the `data` field. */
    pub data_kind: String,

    /** Language-specific metadata for this execution.
     * See ScalaMainClass as an example. */
    pub data: Value,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct DebugSessionAddress {
    /** The Debug Adapter Protocol server's connection uri */
    pub uri: Uri,
}

#[cfg(test)]
mod tests {
    use crate::bsp_types::tests::{test_deserialization, test_serialization};

    use super::*;

    #[test]
    fn debug_session_method() {
        assert_eq!(DebugSession::METHOD, "debugSession/start");
    }

    #[test]
    fn debug_session_params() {
        let test_data = DebugSessionParams {
            targets: vec![BuildTargetIdentifier::default()],
            data_kind: "test_dataKind".to_string(),
            data: serde_json::json!({"dataKey": "dataValue"}),
        };

        test_deserialization(
            r#"{"targets":[{"uri":""}],"dataKind":"test_dataKind","data":{"dataKey":"dataValue"}}"#,
            &test_data,
        );

        let mut modified = test_data.clone();
        modified.targets = vec![];
        test_deserialization(
            r#"{"targets":[],"dataKind":"test_dataKind","data":{"dataKey":"dataValue"}}"#,
            &modified,
        );
    }

    #[test]
    fn debug_session_address() {
        test_serialization(
            &DebugSessionAddress {
                uri: Uri::default(),
            },
            r#"{"uri":""}"#,
        );
    }
}
