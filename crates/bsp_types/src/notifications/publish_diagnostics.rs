use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::notifications::{Location, Notification, Range};
use crate::{BuildTargetIdentifier, OriginId, OtherData, TextDocumentIdentifier, URI};

/// The Diagnostics notification are sent from the server to the client to signal results of validation runs.
///
/// When reset is true, the client must clean all previous diagnostics associated with the same textDocument and
/// buildTarget and set instead the diagnostics in the request. This is the same behaviour as PublishDiagnosticsParams
/// in the LSP. When reset is false, the diagnostics are added to the last active diagnostics, allowing build tools to
/// stream diagnostics to the client.
///
/// It is the server's responsibility to manage the lifetime of the diagnostics by using the appropriate value in the reset field.
/// Clients generate new diagnostics by calling any BSP endpoint that triggers a buildTarget/compile, such as buildTarget/compile, buildTarget/test and buildTarget/run.
///
/// If the computed set of diagnostic is empty, the server must push an empty array with reset set to true, in order to clear previous diagnostics.
///
/// The optional originId field in the definition of PublishDiagnosticsParams can be used by clients to know which request originated the notification.
/// This field will be defined if the client defined it in the original request that triggered this notification.
#[derive(Debug)]
pub enum OnBuildPublishDiagnostics {}

impl Notification for OnBuildPublishDiagnostics {
    type Params = PublishDiagnosticsParams;
    const METHOD: &'static str = "build/publishDiagnostics";
}

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

/// Diagnostic is defined as it is in the LSP.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    /// The range at which the message applies.
    pub range: Range,
    /// The diagnostic's severity. Can be omitted. If omitted it is up to the
    /// client to interpret diagnostics as error, warning, info or hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<DiagnosticSeverity>,
    /// The diagnostic's code, which might appear in the user interface.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<DiagnosticCode>,
    /// An optional property to describe the error code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_description: Option<CodeDescription>,
    /// A human-readable string describing the source of this
    /// diagnostic, e.g. 'typescript' or 'super lint'.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// The diagnostic's message.
    pub message: String,
    /// Additional metadata about the diagnostic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<DiagnosticTag>>,
    /// An array of related diagnostic information, e.g. when symbol-names within
    /// a scope collide all definitions can be marked via this property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_information: Option<Vec<DiagnosticRelatedInformation>>,
    /// A data entry field that is preserved between a
    /// `textDocument/publishDiagnostics` notification and
    /// `textDocument/codeAction` request.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub data: Option<DiagnosticData>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "dataKind", content = "data")]
pub enum NamedDiagnosticData {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DiagnosticData {
    Named(NamedDiagnosticData),
    Other(OtherData),
}

impl DiagnosticData {}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DiagnosticCode {
    String(String),
    I32(i32),
}

/// Structure to capture a description for an error code.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeDescription {
    /// An URI to open with more information about the diagnostic error.
    pub href: URI,
}

/// Represents a related message and source code location for a diagnostic.
/// This should be used to point to code locations that cause or are related to
/// a diagnostics, e.g when duplicating a symbol in a scope.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticRelatedInformation {
    /// The location of this related diagnostic information.
    pub location: Location,
    /// The message of this related diagnostic information.
    pub message: String,
}

#[derive(
    Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize_repr, Deserialize_repr,
)]
#[repr(u8)]
pub enum DiagnosticSeverity {
    #[default]
    /// Reports an error.
    Error = 1,
    /// Reports a warning.
    Warning = 2,
    /// Reports an information.
    Information = 3,
    /// Reports a hint.
    Hint = 4,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DiagnosticTag(pub i32);

impl DiagnosticTag {
    /// Unused or unnecessary code.
    ///
    /// Clients are allowed to render diagnostics with this tag faded out
    /// instead of having an error squiggle.
    pub const UNNECESSARY: DiagnosticTag = DiagnosticTag::new(1);
    /// Deprecated or obsolete code.
    ///
    /// Clients are allowed to rendered diagnostics with this tag strike through.
    pub const DEPRECATED: DiagnosticTag = DiagnosticTag::new(2);

    pub const fn new(tag: i32) -> Self {
        Self(tag)
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
