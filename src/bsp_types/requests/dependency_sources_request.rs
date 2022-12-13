use crate::bsp_types::{BuildTargetIdentifier, MethodName, Uri};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DependencySourcesParams {
    pub targets: Vec<BuildTargetIdentifier>,
}

impl MethodName for DependencySourcesParams {
    fn get_method_name() -> &'static str {
        "buildTarget/dependencySources"
    }
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
