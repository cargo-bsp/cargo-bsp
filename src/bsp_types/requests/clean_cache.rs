use crate::bsp_types::{BuildTargetIdentifier, MethodName};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CleanCacheParams {
    /** The build targets to clean. */
    pub targets: Vec<BuildTargetIdentifier>,
}

impl MethodName for CleanCacheParams {
    fn get_method_name() -> &'static str {
        "buildTarget/cleanCache"
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CleanCacheResult {
    /** Optional message to display to the user. */
    pub message: Option<String>,
    /** Indicates whether the clean cache request was performed or not. */
    pub cleaned: bool,
}
