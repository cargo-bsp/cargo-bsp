use serde::{Deserialize, Serialize};

use crate::bsp_types::requests::Request;
use crate::bsp_types::{BuildTargetIdentifier, Uri};

#[derive(Debug)]
pub enum DependencySources {}

impl Request for DependencySources {
    type Params = DependencySourcesParams;
    type Result = DependencySourcesResult;
    const METHOD: &'static str = "buildTarget/dependencySources";
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DependencySourcesParams {
    pub targets: Vec<BuildTargetIdentifier>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DependencySourcesResult {
    pub items: Vec<DependencySourcesItem>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DependencySourcesItem {
    pub target: BuildTargetIdentifier,
    /** List of resources containing source files of the
     * target's dependencies.
     * Can be source files, jar files, zip files, or directories. */
    pub sources: Vec<Uri>,
}
