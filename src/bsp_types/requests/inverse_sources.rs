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

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct InverseSourcesParams {
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct InverseSourcesResult {
    pub targets: Vec<BuildTargetIdentifier>,
}

#[cfg(test)]
mod tests {
    use crate::bsp_types::tests::{test_deserialization, test_serialization};

    use super::*;

    #[test]
    fn inverse_sources_method() {
        assert_eq!(InverseSources::METHOD, "textDocument/inverseSources");
    }

    #[test]
    fn inverse_sources_params() {
        test_deserialization(
            r#"{"textDocument":{"uri":""}}"#,
            &InverseSourcesParams {
                text_document: TextDocumentIdentifier::default(),
            },
        );
    }

    #[test]
    fn inverse_sources_result() {
        test_serialization(
            &InverseSourcesResult {
                targets: vec![BuildTargetIdentifier::default()],
            },
            r#"{"targets":[{"uri":""}]}"#,
        );
        test_serialization(
            &InverseSourcesResult { targets: vec![] },
            r#"{"targets":[]}"#,
        );
    }
}
