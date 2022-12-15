use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::bsp_types::{BuildClientCapabilities, BuildServerCapabilities, MethodName, Uri};
use crate::bsp_types::requests::Request;

#[derive(Debug)]
pub enum InitializeBuild {}

impl Request for InitializeBuild {
    type Params = InitializeBuildParams;
    type Result = InitializeBuildResult;
    const METHOD: &'static str = "build/initialize";
}

/** Client's initializing request */
#[derive(Debug, Serialize, Deserialize, Default)]
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

impl MethodName for InitializeBuildParams {
    fn get_method_name() -> &'static str {
        "build/initialize"
    }
}

/** Server's response for client's InitializeBuildParams request */
#[derive(Debug, Serialize, Deserialize, Default)]
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
