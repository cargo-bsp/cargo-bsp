use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeBuildParams {
    /// Name of the client
    pub display_name: String,
    /// The version of the client
    pub version: String,
    /// The BSP version that the client speaks
    pub bsp_version: String,
    /// The rootUri of the workspace
    pub root_uri: URI,
    /// The capabilities of the client
    pub capabilities: BuildClientCapabilities,
    /// Additional metadata about the client
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub data: Option<InitializeBuildParamsData>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_deserialization;

    #[test]
    fn initialize_build_params() {
        let test_data = InitializeBuildParams {
            display_name: "test_name".to_string(),
            version: "1.0.0".to_string(),
            bsp_version: "2.0.0".to_string(),
            root_uri: URI::from("file:///test"),
            capabilities: BuildClientCapabilities::default(),
            data: Some(InitializeBuildParamsData::Other(OtherData {
                data_kind: "test_dataKind".to_string(),
                data: serde_json::json!({"dataKey": "dataValue"}),
            })),
        };

        test_deserialization(
            r#"{"displayName":"test_name","version":"1.0.0","bspVersion":"2.0.0","rootUri":"file:///test","capabilities":{"languageIds":[]},"dataKind":"test_dataKind","data":{"dataKey":"dataValue"}}"#,
            &test_data,
        );

        test_deserialization(
            r#"{"displayName":"","version":"","bspVersion":"","rootUri":"","capabilities":{"languageIds":[]}}"#,
            &InitializeBuildParams::default(),
        );
    }
}
