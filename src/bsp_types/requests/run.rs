use crate::bsp_types::{BuildTargetIdentifier, MethodName};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RunParams {
    /** The build target to run. */
    target: BuildTargetIdentifier,

    /** A unique identifier generated by the client to identify this request.
     * The server may include this id in triggered notifications or responses. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<String>,

    /** Optional arguments to the executed application. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<String>>,

    /** Kind of data to expect in the data field. If this field is not set, the kind of data is not specified. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_kind: Option<String>,

    /** Language-specific metadata for this execution.
     * See ScalaMainClass as an example. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl MethodName for RunParams {
    fn get_method_name() -> &'static str {
        "buildTarget/run"
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RunResult {
    /** An optional request id to know the origin of this report. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<String>,

    /** A status code for the execution. */
    pub status_code: i32,
}
