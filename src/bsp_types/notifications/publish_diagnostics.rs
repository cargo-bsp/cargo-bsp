use serde::{Deserialize, Serialize};

use crate::bsp_types::notifications::Notification;
use crate::bsp_types::{BuildTargetIdentifier, TextDocumentIdentifier};

#[derive(Debug)]
pub enum PublishDiagnostics {}

impl Notification for PublishDiagnostics {
    type Params = PublishDiagnosticsParams;
    const METHOD: &'static str = "build/publishDiagnostics";
}

/* Publish Diagnostics notification params */
#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PublishDiagnosticsParams {
    /** The document where the diagnostics are published. */
    pub text_document: TextDocumentIdentifier,

    /** The build target where the diagnostics origin.
     * It is valid for one text document to belong to multiple
     * build targets, for example sources that are compiled against multiple
     * platforms (JVM, JavaScript). */
    pub build_target: BuildTargetIdentifier,

    /** The request id that originated this notification. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<String>,

    /** The diagnostics to be published by the client. */
    pub diagnostics: Vec<Diagnostic>,

    /** Whether the client should clear the previous diagnostics
     * mapped to the same `textDocument` and `buildTarget`. */
    pub reset: bool,
}

pub type Diagnostic = lsp_types::Diagnostic;

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use super::*;

    #[test]
    fn publish_diagnostics_method() {
        assert_eq!(PublishDiagnostics::METHOD, "build/publishDiagnostics");
    }

    #[test]
    fn publish_diagnostics_params() {
        let test_data = PublishDiagnosticsParams {
            text_document: TextDocumentIdentifier::default(),
            build_target: BuildTargetIdentifier::default(),
            origin_id: Some("test_originId".to_string()),
            diagnostics: vec![Diagnostic::default()],
            reset: true,
        };

        assert_json_snapshot!(test_data,
            @r###"
        {
          "textDocument": {
            "uri": ""
          },
          "buildTarget": {
            "uri": ""
          },
          "originId": "test_originId",
          "diagnostics": [
            {
              "range": {
                "start": {
                  "line": 0,
                  "character": 0
                },
                "end": {
                  "line": 0,
                  "character": 0
                }
              },
              "message": ""
            }
          ],
          "reset": true
        }
        "###
        );
        assert_json_snapshot!(PublishDiagnosticsParams::default(),
            @r###"
        {
          "textDocument": {
            "uri": ""
          },
          "buildTarget": {
            "uri": ""
          },
          "diagnostics": [],
          "reset": false
        }
        "###
        );
    }
}
