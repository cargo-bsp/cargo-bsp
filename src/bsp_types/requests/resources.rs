use serde::{Deserialize, Serialize};

use crate::bsp_types::{BuildTargetIdentifier, MethodName, Uri};
use crate::bsp_types::requests::Request;

#[derive(Debug)]
pub enum Resources {}

impl Request for Resources {
    type Params = ResourcesParams;
    type Result = ResourcesResult;
    const METHOD: &'static str = "buildTarget/resources";
}

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
