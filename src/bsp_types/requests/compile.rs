use crate::bsp_types::{BuildTargetIdentifier, MethodName};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/*
NOTE THAT:
Response has error field defined as follows:
error: JSON-RPC code and message set in case an exception happens during the request.
*/

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompileParams {
    /** A sequence of build targets to compile. */
    pub targets: Vec<BuildTargetIdentifier>,

    /** A unique identifier generated by the client to identify this request.
     * The server may include this id in triggered notifications or responses. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<String>,

    /** Optional arguments to the compilation process. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<String>>,
}

impl MethodName for CompileParams {
    fn get_method_name() -> &'static str {
        "buildTarget/compile"
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompileResult {
    /** An optional request id to know the origin of this report. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<String>,

    /** A status code for the execution. */
    pub status_code: i32,

    /** Kind of data to expect in the `data` field. If this field is not set, the kind of data is not specified. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_kind: Option<String>,

    /** A field containing language-specific information, like products
     * of compilation or compiler-specific metadata the client needs to know. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}
