use crate::bsp_types::{BuildTargetIdentifier, MethodName};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct InverseSourcesParams {
    // pub text_document: TextDocumentIdentifier, //todo ask about this
    pub text_document: u32, //something what works for now
}

impl MethodName for InverseSourcesParams {
    fn get_method_name() -> &'static str {
        "textDocument/inverseSources"
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct InverseSourcesResult {
    pub targets: Vec<BuildTargetIdentifier>,
}
