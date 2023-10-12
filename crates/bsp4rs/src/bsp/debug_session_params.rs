use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugSessionParams {
    /// A sequence of build targets affected by the debugging action.
    pub targets: Vec<BuildTargetIdentifier>,
    /// Language-specific metadata for this execution.
    /// See ScalaMainClass as an example.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub data: Option<DebugSessionParamsData>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_deserialization;

    #[test]
    fn debug_session_params() {
        let test_data = DebugSessionParams {
            targets: vec![BuildTargetIdentifier::default()],
            data: Some(DebugSessionParamsData::Other(OtherData {
                data_kind: "test_dataKind".to_string(),
                data: serde_json::json!({"dataKey": "dataValue"}),
            })),
        };

        test_deserialization(
            r#"{"targets":[{"uri":""}],"dataKind":"test_dataKind","data":{"dataKey":"dataValue"}}"#,
            &test_data,
        );

        test_deserialization(r#"{"targets":[]}"#, &DebugSessionParams::default());
    }
}
