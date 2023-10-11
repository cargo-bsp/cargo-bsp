use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeBuildResult {
    /// Name of the server
    pub display_name: String,
    /// The version of the server
    pub version: String,
    /// The BSP version that the server speaks
    pub bsp_version: String,
    /// The capabilities of the build server
    pub capabilities: BuildServerCapabilities,
    /// Additional metadata about the server
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub data: Option<InitializeBuildResultData>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn initialize_build_result() {
        let test_data = InitializeBuildResult {
            display_name: "test_name".to_string(),
            version: "1.0.0".to_string(),
            bsp_version: "2.0.0".to_string(),
            capabilities: BuildServerCapabilities::default(),
            data: Some(InitializeBuildResultData::Other(OtherData {
                data_kind: "test_dataKind".to_string(),
                data: serde_json::json!({"dataKey": "dataValue"}),
            })),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "displayName": "test_name",
          "version": "1.0.0",
          "bspVersion": "2.0.0",
          "capabilities": {},
          "dataKind": "test_dataKind",
          "data": {
            "dataKey": "dataValue"
          }
        }
        "#
        );
        assert_json_snapshot!(InitializeBuildResult::default(),
            @r#"
        {
          "displayName": "",
          "version": "",
          "bspVersion": "",
          "capabilities": {}
        }
        "#
        );
    }
}
