use serde::{Deserialize, Serialize};

use crate::requests::Request;
use crate::{BuildTargetIdentifier, OtherData, URI};

/// The debug request is sent from the client to the server to debug build target(s). The
/// server launches a [Microsoft DAP](https://microsoft.github.io/debug-adapter-protocol/) server
/// and returns a connection URI for the client to interact with.
#[derive(Debug)]
pub enum DebugSessionStart {}

impl Request for DebugSessionStart {
    type Params = DebugSessionParams;
    type Result = DebugSessionAddress;
    const METHOD: &'static str = "debugSession/start";
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugSessionParams {
    /// A sequence of build targets affected by the debugging action.
    pub targets: Vec<BuildTargetIdentifier>,
    /// Language-specific metadata for this execution.
    /// See ScalaMainClass as an example.
    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub data: Option<DebugSessionParamsData>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "dataKind", content = "data")]
pub enum NamedDebugSessionParamsData {
    ScalaTestSuites(Vec<String>),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DebugSessionParamsData {
    Named(NamedDebugSessionParamsData),
    Other(OtherData),
}

impl DebugSessionParamsData {
    pub fn scala_test_suites(data: Vec<String>) -> Self {
        Self::Named(NamedDebugSessionParamsData::ScalaTestSuites(data))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugSessionAddress {
    /// The Debug Adapter Protocol server's connection uri
    pub uri: URI,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use crate::tests::test_deserialization;

    use super::*;

    #[test]
    fn debug_session_method() {
        assert_eq!(DebugSessionStart::METHOD, "debugSession/start");
    }

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
