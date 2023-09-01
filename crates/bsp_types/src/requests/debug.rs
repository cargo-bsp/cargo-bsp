use serde::{Deserialize, Serialize};

use crate::requests::Request;
use crate::{BuildTargetIdentifier, OtherData, URI};

#[derive(Debug)]
pub enum DebugSession {}

impl Request for DebugSession {
    type Params = DebugSessionParams;
    type Result = DebugSessionAddress;
    const METHOD: &'static str = "debugSession/start";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DebugSessionParams {
    /** A sequence of build targets affected by the debugging action. */
    pub targets: Vec<BuildTargetIdentifier>,

    /** Language-specific metadata for this execution.
    See ScalaMainClass as an example. */
    #[serde(flatten)]
    pub data: DebugSessionParamsData,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "dataKind", content = "data")]
pub enum NamedDebugSessionParamsData {
    ScalaTestSuites(Vec<String>),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DebugSessionParamsData {
    Named(NamedDebugSessionParamsData),
    Other(OtherData),
}

impl DebugSessionParamsData {
    pub fn scala_test_suites(data: Vec<String>) -> Self {
        DebugSessionParamsData::Named(NamedDebugSessionParamsData::ScalaTestSuites(data))
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct DebugSessionAddress {
    /** The Debug Adapter Protocol server's connection uri */
    pub uri: URI,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use crate::tests::test_deserialization;

    use super::*;

    #[test]
    fn debug_session_method() {
        assert_eq!(DebugSession::METHOD, "debugSession/start");
    }

    #[test]
    fn debug_session_params() {
        let mut test_data = DebugSessionParams {
            targets: vec![BuildTargetIdentifier::default()],
            data: DebugSessionParamsData::Other(OtherData {
                data_kind: "test_dataKind".to_string(),
                data: serde_json::json!({"dataKey": "dataValue"}),
            }),
        };

        test_deserialization(
            r#"{"targets":[{"uri":""}],"dataKind":"test_dataKind","data":{"dataKey":"dataValue"}}"#,
            &test_data,
        );

        test_data.targets = vec![];
        test_data.data = DebugSessionParamsData::Other(OtherData {
            data_kind: "".to_string(),
            data: serde_json::Value::Null,
        });
        test_deserialization(r#"{"targets":[],"dataKind":"","data":null}"#, &test_data);
    }

    #[test]
    fn debug_session_address() {
        let test_data = DebugSessionAddress {
            uri: "test_uri".into(),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "uri": "test_uri"
        }
        "#
        );
        assert_json_snapshot!(DebugSessionAddress::default(),
            @r#"
        {
          "uri": ""
        }
        "#
        );
    }
}
