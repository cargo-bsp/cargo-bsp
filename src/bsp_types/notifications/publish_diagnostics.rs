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
#[derive(Debug, Serialize, Deserialize, Default)]
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
