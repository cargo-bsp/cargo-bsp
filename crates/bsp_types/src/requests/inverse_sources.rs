use serde::{Deserialize, Serialize};

use crate::requests::Request;
use crate::{BuildTargetIdentifier, TextDocumentIdentifier};

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
    use insta::assert_json_snapshot;

    use crate::tests::test_deserialization;

    use super::*;

    #[test]
    fn inverse_sources_method() {
        assert_eq!(InverseSources::METHOD, "textDocument/inverseSources");
    }

    #[test]
    fn inverse_sources_params() {
        test_deserialization(
            r#"{"textDocument":{"uri":""}}"#,
            &InverseSourcesParams::default(),
        );
    }

    #[test]
    fn inverse_sources_result() {
        let test_data = InverseSourcesResult {
            targets: vec![BuildTargetIdentifier::default()],
        };

        assert_json_snapshot!(test_data,
            @r###"
        {
          "targets": [
            {
              "uri": ""
            }
          ]
        }
        "###
        );
        assert_json_snapshot!(InverseSourcesResult::default(),
            @r###"
        {
          "targets": []
        }
        "###
        );
    }
}
