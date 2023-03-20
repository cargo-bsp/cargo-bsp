use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::bsp_types::requests::Request;
use crate::bsp_types::{BuildClientCapabilities, BuildServerCapabilities, Uri};

#[derive(Debug)]
pub enum InitializeBuild {}

impl Request for InitializeBuild {
    type Params = InitializeBuildParams;
    type Result = InitializeBuildResult;
    const METHOD: &'static str = "build/initialize";
}

/** Client's initializing request */
#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InitializeBuildParams {
    /** Name of the client */
    pub display_name: String,

    /** The version of the client */
    pub version: String,

    /** The BSP version that the client speaks */
    pub bsp_version: String,

    /** The rootUri of the workspace */
    pub root_uri: Uri,

    /** The capabilities of the client */
    pub capabilities: BuildClientCapabilities,

    /** Additional metadata about the client */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/** Server's response for client's InitializeBuildParams request */
#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InitializeBuildResult {
    /** Name of the server */
    pub display_name: String,

    /** The version of the server */
    pub version: String,

    /** The BSP version that the server speaks */
    pub bsp_version: String,

    /** The capabilities of the build server */
    pub capabilities: BuildServerCapabilities,

    /** Additional metadata about the server */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[cfg(test)]
mod tests {
    use crate::bsp_types::tests::{test_deserialization, test_serialization};

    use super::*;

    #[test]
    fn initialize_build_method() {
        assert_eq!(InitializeBuild::METHOD, "build/initialize");
    }

    #[test]
    fn initialize_build_params() {
        let test_data = InitializeBuildParams {
            display_name: "test_name".to_string(),
            version: "1.0.0".to_string(),
            bsp_version: "2.0.0".to_string(),
            root_uri: Uri::from("file:///test"),
            capabilities: BuildClientCapabilities::default(),
            data: Some(serde_json::json!({"dataKey": "dataValue"})),
        };

        test_deserialization(
            r#"{"displayName":"test_name","version":"1.0.0","bspVersion":"2.0.0","rootUri":"file:///test","capabilities":{"languageIds":[]},"data":{"dataKey":"dataValue"}}"#,
            &test_data,
        );

        let mut modified = test_data.clone();
        modified.data = None;
        test_deserialization(
            r#"{"displayName":"test_name","version":"1.0.0","bspVersion":"2.0.0","rootUri":"file:///test","capabilities":{"languageIds":[]}}"#,
            &modified,
        );
    }

    #[test]
    fn initialize_build_result() {
        let test_data = InitializeBuildResult {
            display_name: "test_name".to_string(),
            version: "1.0.0".to_string(),
            bsp_version: "2.0.0".to_string(),
            capabilities: BuildServerCapabilities::default(),
            data: Some(serde_json::json!({"dataKey": "dataValue"})),
        };

        test_serialization(
            &test_data,
            r#"{"displayName":"test_name","version":"1.0.0","bspVersion":"2.0.0","capabilities":{},"data":{"dataKey":"dataValue"}}"#,
        );

        let mut modified = test_data.clone();
        modified.data = None;
        test_serialization(
            &modified,
            r#"{"displayName":"test_name","version":"1.0.0","bspVersion":"2.0.0","capabilities":{}}"#,
        );
    }
}
