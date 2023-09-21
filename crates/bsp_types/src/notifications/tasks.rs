use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::notifications::{Notification, Range, TaskId};
use crate::{BuildTargetIdentifier, Identifier, OtherData, StatusCode, URI};

/// The BSP server can inform the client on the execution state of any task in the
/// build tool. The execution of some tasks, such as compilation or tests, must
/// always be reported by the server.
///
/// The server may also send additional task notifications for actions not covered
/// by the protocol, such as resolution or packaging. BSP clients can then display
/// this information to their users at their discretion.
///
/// When beginning a task, the server may send `build/taskStart`, intermediate
/// updates may be sent in `build/taskProgress`.
///
/// If a `build/taskStart` notification has been sent, the server must send
/// `build/taskFinish` on completion of the same task.
///
/// `build/taskStart`, `build/taskProgress` and `build/taskFinish` notifications for
/// the same task must use the same `taskId`.
///
/// Tasks that are spawned by another task should reference the originating task's
/// `taskId` in their own `taskId`'s `parent` field. Tasks spawned directly by a
/// request should reference the request's `originId` parent.
#[derive(Debug)]
pub enum OnBuildTaskStart {}

impl Notification for OnBuildTaskStart {
    type Params = TaskStartParams;
    const METHOD: &'static str = "build/taskStart";
}

/// After a `taskStart` and before `taskFinish` for a `taskId`, the server may send
/// any number of progress notifications.
#[derive(Debug)]
pub enum OnBuildTaskProgress {}

impl Notification for OnBuildTaskProgress {
    type Params = TaskProgressParams;
    const METHOD: &'static str = "build/taskProgress";
}

/// A `build/taskFinish` notification must always be sent after a `build/taskStart`
/// with the same `taskId` was sent.
#[derive(Debug)]
pub enum OnBuildTaskFinish {}

impl Notification for OnBuildTaskFinish {
    type Params = TaskFinishParams;
    const METHOD: &'static str = "build/taskFinish";
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStartParams {
    /// Unique id of the task with optional reference to parent task id
    pub task_id: TaskId,
    /// Timestamp of when the event started in milliseconds since Epoch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_time: Option<i64>,
    /// Message describing the task.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Optional metadata about the task.
    /// Objects for specific tasks like compile, test, etc are specified in the protocol.
    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub data: Option<TaskStartData>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "dataKind", content = "data")]
pub enum NamedTaskStartData {
    CompileTask(CompileTask),
    TestStart(TestStart),
    TestTask(TestTask),
}

/// Task start notifications may contain an arbitrary interface in their `data`
/// field. The kind of interface that is contained in a notification must be
/// specified in the `dataKind` field.
///
/// There are predefined kinds of objects for compile and test tasks, as described
/// in [[bsp#BuildTargetCompile]] and [[bsp#BuildTargetTest]]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TaskStartData {
    Named(NamedTaskStartData),
    Other(OtherData),
}

impl TaskStartData {
    pub fn compile_task(data: CompileTask) -> Self {
        Self::Named(NamedTaskStartData::CompileTask(data))
    }
    pub fn test_start(data: TestStart) -> Self {
        Self::Named(NamedTaskStartData::TestStart(data))
    }
    pub fn test_task(data: TestTask) -> Self {
        Self::Named(NamedTaskStartData::TestTask(data))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskProgressParams {
    /// Unique id of the task with optional reference to parent task id
    pub task_id: TaskId,
    /// Timestamp of when the event started in milliseconds since Epoch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_time: Option<i64>,
    /// Message describing the task.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// If known, total amount of work units in this task.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
    /// If known, completed amount of work units in this task.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<i64>,
    /// Name of a work unit. For example, "files" or "tests". May be empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// Optional metadata about the task.
    /// Objects for specific tasks like compile, test, etc are specified in the protocol.
    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub data: Option<TaskProgressData>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "dataKind", content = "data")]
pub enum NamedTaskProgressData {}

/// Task progress notifications may contain an arbitrary interface in their `data`
/// field. The kind of interface that is contained in a notification must be
/// specified in the `dataKind` field.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TaskProgressData {
    Named(NamedTaskProgressData),
    Other(OtherData),
}

impl TaskProgressData {}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskFinishParams {
    /// Unique id of the task with optional reference to parent task id
    pub task_id: TaskId,
    /// Timestamp of when the event started in milliseconds since Epoch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_time: Option<i64>,
    /// Message describing the task.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Task completion status.
    pub status: StatusCode,
    /// Optional metadata about the task.
    /// Objects for specific tasks like compile, test, etc are specified in the protocol.
    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub data: Option<TaskFinishData>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "dataKind", content = "data")]
pub enum NamedTaskFinishData {
    CompileReport(CompileReport),
    TestFinish(TestFinish),
    TestReport(TestReport),
}

/// Task finish notifications may contain an arbitrary interface in their `data`
/// field. The kind of interface that is contained in a notification must be
/// specified in the `dataKind` field.
///
/// There are predefined kinds of objects for compile and test tasks, as described
/// in [[bsp#BuildTargetCompile]] and [[bsp#BuildTargetTest]]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TaskFinishData {
    Named(NamedTaskFinishData),
    Other(OtherData),
}

impl TaskFinishData {
    pub fn compile_report(data: CompileReport) -> Self {
        Self::Named(NamedTaskFinishData::CompileReport(data))
    }
    pub fn test_finish(data: TestFinish) -> Self {
        Self::Named(NamedTaskFinishData::TestFinish(data))
    }
    pub fn test_report(data: TestReport) -> Self {
        Self::Named(NamedTaskFinishData::TestReport(data))
    }
}

/// The beginning of a compilation unit may be signalled to the client with a
/// `build/taskStart` notification. When the compilation unit is a build target, the
/// notification's `dataKind` field must be "compile-task" and the `data` field must
/// include a `CompileTask` object:
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileTask {
    pub target: BuildTargetIdentifier,
}

/// The completion of a compilation task should be signalled with a
/// `build/taskFinish` notification. When the compilation unit is a build target,
/// the notification's `dataKind` field must be `compile-report` and the `data`
/// field must include a `CompileReport` object:
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileReport {
    /// The build target that was compiled.
    pub target: BuildTargetIdentifier,
    /// An optional request id to know the origin of this report.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<Identifier>,
    /// The total number of reported errors compiling this target.
    pub errors: i32,
    /// The total number of reported warnings compiling the target.
    pub warnings: i32,
    /// The total number of milliseconds it took to compile the target.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<i64>,
    /// The compilation was a noOp compilation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_op: Option<bool>,
}

/// The beginning of a testing unit may be signalled to the client with a
/// `build/taskStart` notification. When the testing unit is a build target, the
/// notification's `dataKind` field must be `test-task` and the `data` field must
/// include a `TestTask` object.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestTask {
    pub target: BuildTargetIdentifier,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestReport {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<Identifier>,
    /// The build target that was compiled.
    pub target: BuildTargetIdentifier,
    /// The total number of successful tests.
    pub passed: i32,
    /// The total number of failed tests.
    pub failed: i32,
    /// The total number of ignored tests.
    pub ignored: i32,
    /// The total number of cancelled tests.
    pub cancelled: i32,
    /// The total number of skipped tests.
    pub skipped: i32,
    /// The total number of milliseconds tests take to run (e.g. doesn't include compile times).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<i64>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestStart {
    /// Name or description of the test.
    pub display_name: String,
    /// Source location of the test, as LSP location.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestFinish {
    /// Name or description of the test.
    pub display_name: String,
    /// Information about completion of the test, for example an error message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Completion status of the test.
    pub status: TestStatus,
    /// Source location of the test, as LSP location.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    /// Optionally, structured metadata about the test completion.
    /// For example: stack traces, expected/actual values.
    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub data: Option<TestFinishData>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "dataKind", content = "data")]
pub enum NamedTestFinishData {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TestFinishData {
    Named(NamedTestFinishData),
    Other(OtherData),
}

impl TestFinishData {}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub uri: URI,
    pub range: Range,
}

#[derive(
    Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize_repr, Deserialize_repr,
)]
#[repr(u8)]
pub enum TestStatus {
    #[default]
    /// The test passed successfully.
    Passed = 1,
    /// The test failed.
    Failed = 2,
    /// The test was marked as ignored.
    Ignored = 3,
    /// The test execution was cancelled.
    Cancelled = 4,
    /// The was not included in execution.
    Skipped = 5,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use super::*;

    #[test]
    fn task_start_method() {
        assert_eq!(OnBuildTaskStart::METHOD, "build/taskStart");
    }

    #[test]
    fn task_progress_method() {
        assert_eq!(OnBuildTaskProgress::METHOD, "build/taskProgress");
    }

    #[test]
    fn task_finish_method() {
        assert_eq!(OnBuildTaskFinish::METHOD, "build/taskFinish");
    }

    #[test]
    fn task_start_params() {
        let test_data = TaskStartParams {
            task_id: TaskId::default(),
            event_time: Some(1),
            message: Some("test_message".to_string()),
            data: Some(TaskStartData::compile_task(CompileTask::default())),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "taskId": {
            "id": ""
          },
          "eventTime": 1,
          "message": "test_message",
          "dataKind": "compile-task",
          "data": {
            "target": {
              "uri": ""
            }
          }
        }
        "#
        );
        assert_json_snapshot!(TaskStartParams::default(),
            @r#"
        {
          "taskId": {
            "id": ""
          }
        }
        "#
        );
    }

    #[test]
    fn task_progress_params() {
        let test_data = TaskProgressParams {
            task_id: TaskId::default(),
            event_time: Some(1),
            message: Some("test_message".to_string()),
            total: Some(2),
            progress: Some(3),
            unit: Some("test_unit".to_string()),
            data: Some(TaskProgressData::Other(OtherData {
                data_kind: "test_dataKind".to_string(),
                data: serde_json::json!({"dataKey": "dataValue"}),
            })),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "taskId": {
            "id": ""
          },
          "eventTime": 1,
          "message": "test_message",
          "total": 2,
          "progress": 3,
          "unit": "test_unit",
          "dataKind": "test_dataKind",
          "data": {
            "dataKey": "dataValue"
          }
        }
        "#
        );
        assert_json_snapshot!(TaskProgressParams::default(),
            @r#"
        {
          "taskId": {
            "id": ""
          }
        }
        "#
        );
    }

    #[test]
    fn task_finish_params() {
        let test_data = TaskFinishParams {
            task_id: TaskId::default(),
            event_time: Some(1),
            message: Some("test_message".to_string()),
            status: StatusCode::default(),
            data: Some(TaskFinishData::compile_report(CompileReport::default())),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "taskId": {
            "id": ""
          },
          "eventTime": 1,
          "message": "test_message",
          "status": 1,
          "dataKind": "compile-report",
          "data": {
            "target": {
              "uri": ""
            },
            "errors": 0,
            "warnings": 0
          }
        }
        "#
        );
        assert_json_snapshot!(TaskFinishParams::default(),
            @r#"
        {
          "taskId": {
            "id": ""
          },
          "status": 1
        }
        "#
        );
    }

    #[test]
    fn task_start_data() {
        assert_json_snapshot!(TaskStartData::compile_task(CompileTask::default()),
            @r#"
        {
          "dataKind": "compile-task",
          "data": {
            "target": {
              "uri": ""
            }
          }
        }
        "#
        );
        assert_json_snapshot!(TaskStartData::test_task(TestTask::default()),
            @r#"
        {
          "dataKind": "test-task",
          "data": {
            "target": {
              "uri": ""
            }
          }
        }
        "#
        );
        assert_json_snapshot!(TaskStartData::test_start(TestStart::default()),
            @r#"
        {
          "dataKind": "test-start",
          "data": {
            "displayName": ""
          }
        }
        "#
        );
    }

    #[test]
    fn task_finish_data() {
        assert_json_snapshot!(TaskFinishData::compile_report(CompileReport::default()),
            @r#"
        {
          "dataKind": "compile-report",
          "data": {
            "target": {
              "uri": ""
            },
            "errors": 0,
            "warnings": 0
          }
        }
        "#
        );
        assert_json_snapshot!(TaskFinishData::test_report(TestReport::default()),
            @r#"
        {
          "dataKind": "test-report",
          "data": {
            "target": {
              "uri": ""
            },
            "passed": 0,
            "failed": 0,
            "ignored": 0,
            "cancelled": 0,
            "skipped": 0
          }
        }
        "#
        );
        assert_json_snapshot!(TaskFinishData::test_finish(TestFinish::default()),
            @r#"
        {
          "dataKind": "test-finish",
          "data": {
            "displayName": "",
            "status": 1
          }
        }
        "#
        );
    }

    #[test]
    fn compile_task_data() {
        assert_json_snapshot!(CompileTask::default(),
            @r#"
        {
          "target": {
            "uri": ""
          }
        }
        "#
        );
    }

    #[test]
    fn compile_report_data() {
        let test_data = CompileReport {
            target: BuildTargetIdentifier::default(),
            origin_id: Some("test_originId".into()),
            errors: 1,
            warnings: 2,
            time: Some(3),
            no_op: Some(true),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "target": {
            "uri": ""
          },
          "originId": "test_originId",
          "errors": 1,
          "warnings": 2,
          "time": 3,
          "noOp": true
        }
        "#
        );
        assert_json_snapshot!(CompileReport::default(),
            @r#"
        {
          "target": {
            "uri": ""
          },
          "errors": 0,
          "warnings": 0
        }
        "#
        );
    }

    #[test]
    fn test_task_data() {
        assert_json_snapshot!(TestTask::default(),
            @r#"
        {
          "target": {
            "uri": ""
          }
        }
        "#
        );
    }

    #[test]
    fn test_report_data() {
        let test_data = TestReport {
            origin_id: None,
            target: BuildTargetIdentifier::default(),
            passed: 1,
            failed: 2,
            ignored: 3,
            cancelled: 4,
            skipped: 5,
            time: Some(6),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "target": {
            "uri": ""
          },
          "passed": 1,
          "failed": 2,
          "ignored": 3,
          "cancelled": 4,
          "skipped": 5,
          "time": 6
        }
        "#
        );
        assert_json_snapshot!(TestReport::default(),
            @r#"
        {
          "target": {
            "uri": ""
          },
          "passed": 0,
          "failed": 0,
          "ignored": 0,
          "cancelled": 0,
          "skipped": 0
        }
        "#
        );
    }

    #[test]
    fn test_start_data() {
        let test_data = TestStart {
            display_name: "test_name".to_string(),
            location: Some(Location {
                uri: "file:///test".into(),
                range: Range::default(),
            }),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "displayName": "test_name",
          "location": {
            "uri": "file:///test",
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            }
          }
        }
        "#
        );
        assert_json_snapshot!(TestStart::default(),
            @r#"
        {
          "displayName": ""
        }
        "#
        );
    }

    #[test]
    fn test_finish_data() {
        let test_data = TestFinish {
            display_name: "test_name".to_string(),
            message: Some("test_message".to_string()),
            status: TestStatus::default(),
            location: Some(Location {
                uri: "file:///test".into(),
                range: Range::default(),
            }),
            data: Some(TestFinishData::Other(OtherData {
                data_kind: "test_dataKind".to_string(),
                data: serde_json::json!({"dataKey": "dataValue"}),
            })),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "displayName": "test_name",
          "message": "test_message",
          "status": 1,
          "location": {
            "uri": "file:///test",
            "range": {
              "start": {
                "line": 0,
                "character": 0
              },
              "end": {
                "line": 0,
                "character": 0
              }
            }
          },
          "dataKind": "test_dataKind",
          "data": {
            "dataKey": "dataValue"
          }
        }
        "#
        );
        assert_json_snapshot!(TestFinish::default(),
            @r#"
        {
          "displayName": "",
          "status": 1
        }
        "#
        );
    }

    #[test]
    fn test_status() {
        assert_json_snapshot!(TestStatus::Passed, @"1");
        assert_json_snapshot!(TestStatus::Failed, @"2");
        assert_json_snapshot!(TestStatus::Ignored, @"3");
        assert_json_snapshot!(TestStatus::Cancelled, @"4");
        assert_json_snapshot!(TestStatus::Skipped, @"5");
    }
}
