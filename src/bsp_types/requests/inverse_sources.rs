use serde::{Deserialize, Serialize};

use crate::bsp_types::requests::Request;
use crate::bsp_types::{BuildTargetIdentifier, TextDocumentIdentifier};

#[derive(Debug)]
pub enum InverseSources {}

impl Request for InverseSources {
    type Params = InverseSourcesParams;
    type Result = InverseSourcesResult;
    const METHOD: &'static str = "textDocument/inverseSources";
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct InverseSourcesParams {
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct InverseSourcesResult {
    pub targets: Vec<BuildTargetIdentifier>,
}
