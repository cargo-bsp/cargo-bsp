use crate::bsp_types::{BuildTargetIdentifier, MethodName, Uri};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesParams {
    pub targets: Vec<BuildTargetIdentifier>,
}

impl MethodName for ResourcesParams {
    fn get_method_name() -> &'static str {
        "buildTarget/resources"
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesResult {
    pub items: Vec<ResourcesItem>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesItem {
    pub target: BuildTargetIdentifier,
    /** List of resource files. */
    pub resources: Vec<Uri>,
}
