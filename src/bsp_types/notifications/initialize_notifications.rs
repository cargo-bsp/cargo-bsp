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

/* Exit Build Notification params */
//dev: same as shutdown build request params are null -
// can be implemented using wrapper like in lsp_types crate
/*
Like the language server protocol, a notification to ask the server to exit its process.
The server should exit with success code 0 if the shutdown request has been received before; otherwise with error code 1.

Notification:
method: build/exit
params: null
 */

//temporary solution, can't implement a trait
pub const EXIT_BUILD_METHOD_NAME: &str = "build/exit";
