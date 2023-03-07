use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::bsp_types::notifications::{Notification, StatusCode, TaskId};
use crate::bsp_types::{BuildTargetIdentifier, OriginId};

#[derive(Debug)]
pub enum TaskStart {}

impl Notification for TaskStart {
    type Params = TaskStartParams;
    const METHOD: &'static str = "build/taskStart";
}

#[derive(Debug)]
pub enum TaskProgress {}

impl Notification for TaskProgress {
    type Params = TaskProgressParams;
    const METHOD: &'static str = "build/taskProgress";
}

#[derive(Debug)]
pub enum TaskFinish {}

impl Notification for TaskFinish {
    type Params = TaskFinishParams;
    const METHOD: &'static str = "build/taskFinish";
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

    /** Serializes to:
     * dataKind: String,
     * data: Option<Value>
     * Where dataKind is: kind of data to expect in the `data` field. If this field is not set,
     * the kind of data is not specified. Kind names for specific tasks like compile, test,
     * etc are specified in the protocol. Data kind options specified in task_data_kind module
     *
     * and data is: Optional metadata about the task. Objects for specific tasks like compile, test,
     * etc are specified in the protocol. */
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub data: Option<TestDataWithKind>,
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

    /** Serializes to:
     * dataKind: String,
     * data: Option<Value>
     * Where dataKind is: kind of data to expect in the `data` field. If this field is not set,
     * the kind of data is not specified. Kind names for specific tasks like compile, test,
     * etc are specified in the protocol. Data kind options specified in task_data_kind module
     *
     * and data is: Optional metadata about the task. Objects for specific tasks like compile, test,
     * etc are specified in the protocol. */
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub data: Option<TestDataWithKind>,
}

#[derive(Debug, Serialize, Deserialize)]
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

    /** Serializes to:
     * dataKind: String,
     * data: Option<Value>
     * Where dataKind is: kind of data to expect in the `data` field. If this field is not set,
     * the kind of data is not specified. Kind names for specific tasks like compile, test,
     * etc are specified in the protocol. Data kind options specified in task_data_kind module
     *
     * and data is: Optional metadata about the task. Objects for specific tasks like compile, test,
     * etc are specified in the protocol. */
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub data: Option<TestDataWithKind>,
}

//dev:: change data field in TaskStartParams, TaskProgressParams, TaskFinishParams when we want to add possibility to send data without datakind
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataWithOptionalDataKind {
    DataWithKind(TestDataWithKind),
    JustData { data: Value },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "dataKind", content = "data")]
pub enum TestDataWithKind {
    CompileTask(CompileTaskData),
    CompileReport(CompileReportData),
    TestTask(TestTaskData),
    TestReport(TestReportData),
    TestStart(TestStartData),
    TestFinish(TestFinishData),
}

/* The beginning of a compilation unit may be signalled to the client with a build/taskStart
 * notification. When the compilation unit is a build target, the notification's dataKind field
 * must be "compile-task" and the data field must include a CompileTask object. */
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompileTaskData {
    pub target: BuildTargetIdentifier,
}

/* The completion of a compilation task should be signalled with a build/taskFinish notification.
 * When the compilation unit is a build target, the notification's dataKind field must be
 * compile-report and the data field must include a CompileReport object. */
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompileReportData {
    /** The build target that was compiled. */
    pub target: BuildTargetIdentifier,

    /** An optional request id to know the origin of this report. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<OriginId>,

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
pub struct TestTaskData {
    pub target: BuildTargetIdentifier,
}

/* The completion of a test task should be signalled with a build/taskFinish notification.
 * When the testing unit is a build target, the notification's dataKind field must be
 * test-report and the data field must include a TestReport object. */
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TestReportData {
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestStartData {
    /** Name or description of the test. */
    pub display_name: String,

    /** Source location of the test, as LSP location. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestFinishData {
    /** Name or description of the test. */
    pub display_name: String,

    /** Information about completion of the test, for example an error message. */
    pub message: Option<String>,

    /** Completion status of the test. */
    pub status: TestStatus,

    /** Source location of the test, as LSP location. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,

    /** Kind of data to expect in the `data` field. If this field is not set, the kind of data is not specified. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_kind: Option<String>,

    /** Optionally, structured metadata about the test completion.
     * For example: stack traces, expected/actual values. */
    pub data: Option<Value>,
}

pub type Location = lsp_types::Location;

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
