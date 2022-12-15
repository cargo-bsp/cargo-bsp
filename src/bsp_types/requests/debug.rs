use crate::bsp_types::{BuildTargetIdentifier, MethodName, Uri};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Default)]
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

impl MethodName for DebugSessionParams {
    fn get_method_name() -> &'static str {
        "debugSession/start"
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DebugSessionAddress {
    /** The Debug Adapter Protocol server's connection uri */
    pub uri: Uri,
}
