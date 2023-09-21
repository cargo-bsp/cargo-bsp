use serde::{Deserialize, Serialize};

use crate::requests::Request;
use crate::{BuildTargetIdentifier, TextDocumentIdentifier};

/// The inverse sources request is sent from the client to the server to query for
/// the list of build targets containing a text document. The server communicates
/// during the initialize handshake whether this method is supported or not. This
/// request can be viewed as the inverse of `buildTarget/sources`, except it only
/// works for text documents and not directories.
#[derive(Debug)]
pub enum BuildTargetInverseSources {}

impl Request for BuildTargetInverseSources {
    type Params = InverseSourcesParams;
    type Result = InverseSourcesResult;
    const METHOD: &'static str = "buildTarget/inverseSources";
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InverseSourcesParams {
    pub text_document: TextDocumentIdentifier,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
        assert_eq!(
            BuildTargetInverseSources::METHOD,
            "buildTarget/inverseSources"
        );
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
            @r#"
        {
          "targets": [
            {
              "uri": ""
            }
          ]
        }
        "#
        );
        assert_json_snapshot!(InverseSourcesResult::default(),
            @r#"
        {
          "targets": []
        }
        "#
        );
    }
}
