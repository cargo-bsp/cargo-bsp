use serde::{Deserialize, Serialize};

use crate::bsp_types::requests::Request;
use crate::bsp_types::BuildTarget;

#[derive(Debug)]
pub enum WorkspaceBuildTargets {}

impl Request for WorkspaceBuildTargets {
    type Params = ();
    type Result = WorkspaceBuildTargetsResult;
    const METHOD: &'static str = "workspace/buildTargets";
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceBuildTargetsParams {}

/** The result of the workspace/buildTargets request */
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceBuildTargetsResult {
    /** The build targets in this workspace that
     * contain sources with the given language ids. */
    pub targets: Vec<BuildTarget>,
}
