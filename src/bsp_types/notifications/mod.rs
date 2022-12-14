use crate::bsp_types::{BuildTargetIdentifier, MethodName, TextDocumentIdentifier};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};

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

#[derive(Debug, Serialize_repr, Deserialize_repr, Default)]
#[repr(u8)]
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
Like the language server protocol, a notification to ask the server to exit its process.
The server should exit with success code 0 if the shutdown request has been received before; otherwise with error code 1.

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

#[derive(Debug, Serialize_repr, Deserialize_repr, Default)]
#[repr(u8)]
pub enum BuildTargetEventKind {
    /** The build target is new. */
    #[default]
    Created = 1,
    /** The build target has changed. */
    Changed = 2,
    /** The build target has been deleted. */
    Deleted = 3,
}

impl MethodName for DidChangeBuildTarget {
    fn get_method_name() -> &'static str {
        "buildTarget/didChange"
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskStartParams {
    /** Unique id of the task with optional reference to parent task id */
    pub task_id: TaskId,

    /** Timestamp of when the event started in milliseconds since Epoch. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_time: Option<i64>,

    /** Message describing the task. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /** Kind of data to expect in the `data` field. If this field is not set, the kind of data is not specified.
     * Kind names for specific tasks like compile, test, etc are specified in the protocol. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_kind: Option<String>,

    /** Optional metadata about the task.
     * Objects for specific tasks like compile, test, etc are specified in the protocol. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl MethodName for TaskStartParams {
    fn get_method_name() -> &'static str {
        "build/taskStart"
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskProgressParams {
    /** Unique id of the task with optional reference to parent task id */
    pub task_id: TaskId,

    /** Timestamp of when the progress event was generated in milliseconds since Epoch. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_time: Option<i64>,

    /** Message describing the task progress.
     * Information about the state of the task at the time the event is sent. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /** If known, total amount of work units in this task. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,

    /** If known, completed amount of work units in this task. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<i64>,

    /** Name of a work unit. For example, "files" or "tests". May be empty. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,

    /** Kind of data to expect in the `data` field. If this field is not set, the kind of data is not specified.
     * Kind names for specific tasks like compile, test, etc are specified in the protocol.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_kind: Option<String>,

    /** Optional metadata about the task.
     * Objects for specific tasks like compile, test, etc are specified in the protocol.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl MethodName for TaskProgressParams {
    fn get_method_name() -> &'static str {
        "build/taskProgress"
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskFinishParams {
    /** Unique id of the task with optional reference to parent task id */
    pub task_id: TaskId,

    /** Timestamp of the event in milliseconds. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_time: Option<i64>,

    /** Message describing the finish event. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /** Task completion status. */
    pub status: StatusCode,

    /** Kind of data to expect in the `data` field. If this field is not set, the kind of data is not specified.
     * Kind names for specific tasks like compile, test, etc are specified in the protocol.
     * Data kind options specified in task_data_kind module */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_kind: Option<String>,

    /** Optional metadata about the task.
     * Objects for specific tasks like compile, test, etc are specified in the protocol. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl MethodName for TaskFinishParams {
    fn get_method_name() -> &'static str {
        "build/taskFinish"
    }
}

pub mod task_data_kind {
    /** `data` field must contain a CompileTask object. */
    pub const COMPILE_TASK: &str = "compile-task";

    /** `data` field must contain a CompileReport object. */
    pub const COMPILE_REPORT: &str = "compile-report";

    /** `data` field must contain a TestTask object. */
    pub const TEST_TASK: &str = "test-task";

    /** `data` field must contain a TestReport object. */
    pub const TEST_REPORT: &str = "test-report";

    /** `data` field must contain a TestStart object. */
    pub const TEST_START: &str = "test-start";

    /** `data` field must contain a TestFinish object. */
    pub const TEST_FINISH: &str = "test-finish";
}

/* The beginning of a compilation unit may be signalled to the client with a build/taskStart
 * notification. When the compilation unit is a build target, the notification's dataKind field
 * must be "compile-task" and the data field must include a CompileTask object. */
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompileTask {
    pub target: BuildTargetIdentifier,
}

/* The completion of a compilation task should be signalled with a build/taskFinish notification.
 * When the compilation unit is a build target, the notification's dataKind field must be
 * compile-report and the data field must include a CompileReport object. */
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompileReport {
    /** The build target that was compiled. */
    pub target: BuildTargetIdentifier,

    /** An optional request id to know the origin of this report. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<String>,

    /** The total number of reported errors compiling this target. */
    pub errors: i32,

    /** The total number of reported warnings compiling the target. */
    pub warnings: i32,

    /** The total number of milliseconds it took to compile the target. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<i32>,

    /** The compilation was a noOp compilation. */
    pub no_op: bool,
}

/* The beginning of a testing unit may be signalled to the client with a build/taskStart notification.
 * When the testing unit is a build target, the notification's dataKind field must be
 * test-task and the data field must include a TestTask object. */
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TestTask {
    pub target: BuildTargetIdentifier,
}

/* The completion of a test task should be signalled with a build/taskFinish notification.
 * When the testing unit is a build target, the notification's dataKind field must be
 * test-report and the data field must include a TestReport object. */
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TestReport {
    /** The build target that was compiled. */
    pub target: BuildTargetIdentifier,

    /** The total number of successful tests. */
    pub passed: i32,

    /** The total number of failed tests. */
    pub failed: i32,

    /** The total number of ignored tests. */
    pub ignored: i32,

    /** The total number of cancelled tests. */
    pub cancelled: i32,

    /** The total number of skipped tests. */
    pub skipped: i32,

    /** The total number of milliseconds tests take to run (e.g. doesn't include compile times). */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<i32>,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, Default)]
#[repr(u8)]
pub enum TestStatus {
    /** The test was successful. */
    Passed = 1,
    /** The test failed. */
    #[default]
    Failed = 2,
    /** The test was ignored. */
    Ignored = 3,
    /** The test was cancelled. */
    Cancelled = 4,
    /** The test was skipped. */
    Skipped = 5,
}
