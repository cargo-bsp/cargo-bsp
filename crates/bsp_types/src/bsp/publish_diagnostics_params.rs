use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishDiagnosticsParams {
    /// The document where the diagnostics are published.
    pub text_document: TextDocumentIdentifier,
    /// The build target where the diagnostics origin.
    /// It is valid for one text document to belong to multiple
    /// build targets, for example sources that are compiled against multiple
    /// platforms (JVM, JavaScript).
    pub build_target: BuildTargetIdentifier,
    /// The request id that originated this notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<OriginId>,
    /// The diagnostics to be published by the client.
    pub diagnostics: Vec<Diagnostic>,
    /// Whether the client should clear the previous diagnostics
    /// mapped to the same `textDocument` and `buildTarget`.
    pub reset: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn publish_diagnostics_params() {
        let test_data = PublishDiagnosticsParams {
            text_document: TextDocumentIdentifier::default(),
            build_target: BuildTargetIdentifier::default(),
            origin_id: Some("test_originId".into()),
            diagnostics: vec![Diagnostic::default()],
            reset: true,
        };

        assert_json_snapshot!(test_data,
            @r#"
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
        "#
        );
        assert_json_snapshot!(PublishDiagnosticsParams::default(),
            @r#"
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
        "#
        );
    }
}
