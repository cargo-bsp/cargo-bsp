use crate::bsp_types::{BuildTargetIdentifier, MethodName, TextDocumentIdentifier};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/* Included in notifications of tasks or requests to signal the completion state. */
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum StatusCode {
    /** Execution was successful. */
    Ok = 1,
    /** Execution failed. */
    #[default]
    Error = 2,
    /** Execution was cancelled. */
    Cancelled = 3,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskId {
    /** A unique identifier */
    pub id: String,

    /** The parent task ids, if any. A non-empty parents field means
        * this task is a sub-task of every parent task id. The child-parent
        * relationship of tasks makes it possible to render tasks in
        * a tree-like user interface or inspect what caused a certain task
        * execution. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parents: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum MessageType {
    #[default]
    Error = 1,
    Warning = 2,
    Info = 3,
    Log = 4,
}

/* Initialized Build notification params */
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct InitializedBuildParams {}

impl MethodName for InitializedBuildParams {
    fn get_method_name() -> &'static str {
        "build/initialized"
    }
}

/* Exit Build Notification params */
//dev: same as shutdown build request params are null -
// can be implemented using wrapper like in lsp_types crate
/*
Like the language server protocol, a notification to ask the server to exit its process. The server should exit with success code 0 if the shutdown request has been received before; otherwise with error code 1.

Notification:
method: build/exit
params: null
 */

//temporary solution, can't implement a trait
pub const EXIT_BUILD_METHOD_NAME: &str = "build/exit";

/* Show message notification */
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ShowMessageParams {
    /** The message type. See {@link MessageType}. */
    #[serde(rename = "type")]
    pub message_type: MessageType,

    /** The task id if any. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<TaskId>,

    /** The request id that originated this notification. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<String>,

    /** The actual message. */
    pub message: String,
}

impl MethodName for ShowMessageParams {
    fn get_method_name() -> &'static str {
        "build/showMessage"
    }
}

/* Log message notification params */
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LogMessageParams {
    /** The message type. See {@link MessageType}. */
    #[serde(rename = "type")]
    pub message_type: MessageType,

    /** The task id if any. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<TaskId>,

    /** The request id that originated this notification. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<String>,

    /** The actual message. */
    pub message: String,
}

impl MethodName for LogMessageParams {
    fn get_method_name() -> &'static str {
        "build/logMessage"
    }
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
    pub diagnostics: Vec<i32>, //TODO Vec<Diagnostic>

    /** Whether the client should clear the previous diagnostics
        * mapped to the same `textDocument` and `buildTarget`. */
    pub reset: bool,
}

impl MethodName for PublishDiagnosticsParams {
    fn get_method_name() -> &'static str {
        "build/publishDiagnostics"
    }
}

/* Build Target Changed Notification params */

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DidChangeBuildTarget {
    pub changes: Vec<BuildTargetEvent>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BuildTargetEvent {
    /** The identifier for the changed build target */
    pub target: BuildTargetIdentifier,

    /** The kind of change for this build target */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<BuildTargetEventKind>,

    /** Any additional metadata about what information changed. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

//todo serialize BuildTargetEventKind and MessageType as numbers

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum BuildTargetEventKind {
    /** The build target is new. */
    #[default]
    Created = 1,
    /** The build target has changed. */
    Changed = 2,
    /** The build target has been deleted. */
    Deleted = 3,
}
