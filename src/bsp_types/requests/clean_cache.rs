use serde::{Deserialize, Serialize};

use crate::bsp_types::requests::Request;
use crate::bsp_types::BuildTargetIdentifier;

#[derive(Debug)]
pub enum CleanCache {}

impl Request for CleanCache {
    type Params = CleanCacheParams;
    type Result = CleanCacheResult;
    const METHOD: &'static str = "buildTarget/cleanCache";
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CleanCacheParams {
    /** The build targets to clean. */
    pub targets: Vec<BuildTargetIdentifier>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CleanCacheResult {
    /** Optional message to display to the user. */
    pub message: Option<String>,
    /** Indicates whether the clean cache request was performed or not. */
    pub cleaned: bool,
}
