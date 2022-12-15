use crate::bsp_types::{BuildTargetIdentifier, MethodName};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DependencyModulesParams {
    pub targets: Vec<BuildTargetIdentifier>,
}

impl MethodName for DependencyModulesParams {
    fn get_method_name() -> &'static str {
        "buildTarget/dependencyModules"
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DependencyModulesResult {
    pub items: Vec<DependencyModulesItem>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DependencyModulesItem {
    pub target: BuildTargetIdentifier,

    pub modules: Vec<DependencyModule>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DependencyModule {
    /** Module name */
    pub name: String,

    /** Module version */
    pub version: String,

    /** Kind of data to expect in the `data` field. If this field is not set, the kind of data is not specified. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_kind: Option<String>,

    /** Language-specific metadata about this module.
     * See MavenDependencyModule as an example. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}
