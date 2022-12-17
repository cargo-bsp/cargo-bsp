use crate::bsp_types::MethodName;
use serde::{Deserialize, Serialize};

/* Initialized Build notification params */
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct InitializedBuildParams {}

impl MethodName for InitializedBuildParams {
    fn get_method_name() -> &'static str {
        "build/initialized"
    }
}

