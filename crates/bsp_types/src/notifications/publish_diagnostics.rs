use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::notifications::{Location, Notification, Range};
use crate::{BuildTargetIdentifier, OtherData, TextDocumentIdentifier, URI};

#[derive(Debug)]
pub enum OnBuildPublishDiagnostics {}

impl Notification for OnBuildPublishDiagnostics {
    type Params = PublishDiagnosticsParams;
    const METHOD: &'static str = "build/publishDiagnostics";
}

/** Publish Diagnostics notification params */
#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PublishDiagnosticsParams {
    /** The document where the diagnostics are published. */
    pub text_document: TextDocumentIdentifier,

    /** The build target where the diagnostics origin.
    It is valid for one text document to belong to multiple
    build targets, for example sources that are compiled against multiple
    platforms (JVM, JavaScript). */
    pub build_target: BuildTargetIdentifier,

    /** The request id that originated this notification. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<String>,

    /** The diagnostics to be published by the client. */
    pub diagnostics: Vec<Diagnostic>,

    /** Whether the client should clear the previous diagnostics
    mapped to the same `textDocument` and `buildTarget`. */
    pub reset: bool,
}

/** Represents a diagnostic, such as a compiler error or warning.
Diagnostic objects are only valid in the scope of a resource. */
#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    /** The range at which the message applies. */
    pub range: Range,
    /** The diagnostic's severity. Can be omitted. If omitted it is up to the
    client to interpret diagnostics as error, warning, info or hint. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<DiagnosticSeverity>,
    /** The diagnostic's code, which might appear in the user interface. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /** An optional property to describe the error code. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_description: Option<CodeDescription>,
    /** A human-readable string describing the source of this
    diagnostic, e.g. 'typescript' or 'super lint'. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /** The diagnostic's message. */
    pub message: String,
    /** An array of related diagnostic information, e.g. when symbol-names within
    a scope collide all definitions can be marked via this property. */
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_information: Vec<DiagnosticRelatedInformation>,
    /** Additional metadata about the diagnostic. */
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<DiagnosticTag>,
    /** A data entry field that is preserved between a `textDocument/publishDiagnostics` notification
    and a `textDocument/codeAction` request. */
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub data: Option<DiagnosticData>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "dataKind", content = "data")]
pub enum NamedDiagnosticData {}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DiagnosticData {
    Named(NamedDiagnosticData),
    Other(OtherData),
}

impl DiagnosticData {}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeDescription {
    pub href: URI,
}

/** Represents a related message and source code location for a diagnostic.
This should be used to point to code locations that cause or are related to
a diagnostics, e.g when duplicating a symbol in a scope. */
#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticRelatedInformation {
    /** The location of this related diagnostic information. */
    pub location: Location,
    /** The message of this related diagnostic information. */
    pub message: String,
}

#[derive(Debug, PartialEq, Clone, Default, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum DiagnosticSeverity {
    #[default]
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DiagnosticTag(pub i32);
impl DiagnosticTag {
    /** Unused or unnecessary code.

    Clients are allowed to render diagnostics with this tag faded out instead of having an error squiggle. */
    pub const UNNECESSARY: DiagnosticTag = DiagnosticTag::new(1);
    /** Deprecated or obsolete code.

    Clients are allowed to rendered diagnostics with this tag strike through. */
    pub const DEPRECATED: DiagnosticTag = DiagnosticTag::new(2);

    pub const fn new(tag: i32) -> Self {
        DiagnosticTag(tag)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use super::*;

    #[test]
    fn publish_diagnostics_method() {
        assert_eq!(
            OnBuildPublishDiagnostics::METHOD,
            "build/publishDiagnostics"
        );
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
