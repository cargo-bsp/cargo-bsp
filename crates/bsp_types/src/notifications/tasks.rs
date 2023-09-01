use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::notifications::{Notification, Range, TaskId};
use crate::{BuildTargetIdentifier, OtherData, StatusCode, URI};

#[derive(Debug)]
pub enum OnBuildTaskStart {}

impl Notification for OnBuildTaskStart {
    type Params = TaskStartParams;
    const METHOD: &'static str = "build/taskStart";
}

#[derive(Debug)]
pub enum OnBuildTaskProgress {}

impl Notification for OnBuildTaskProgress {
    type Params = TaskProgressParams;
    const METHOD: &'static str = "build/taskProgress";
}

#[derive(Debug)]
pub enum OnBuildTaskFinish {}

impl Notification for OnBuildTaskFinish {
    type Params = TaskFinishParams;
    const METHOD: &'static str = "build/taskFinish";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
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

    Where dataKind is: kind of data to expect in the `data` field. If this field is not set,
    the kind of data is not specified. Kind names for specific tasks like compile, test,
    etc are specified in the protocol. Data kind options specified in task_data_kind module
    and data is: Optional metadata about the task. Objects for specific tasks like compile, test,
    etc are specified in the protocol. */
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub data: Option<TaskStartData>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "dataKind", content = "data")]
pub enum NamedTaskStartData {
    CompileTask(CompileTask),
    TestStart(TestStart),
    TestTask(TestTask),
}

/** Task start notifications may contain an arbitrary interface in their `data`
field. The kind of interface that is contained in a notification must be
specified in the `dataKind` field.

There are predefined kinds of objects for compile and test tasks, as described
in [[bsp#BuildTargetCompile]] and [[bsp#BuildTargetTest]] */
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TaskStartData {
    Named(NamedTaskStartData),
    Other(OtherData),
}

impl TaskStartData {
    pub fn compile_task(data: CompileTask) -> Self {
        TaskStartData::Named(NamedTaskStartData::CompileTask(data))
    }
    pub fn test_start(data: TestStart) -> Self {
        TaskStartData::Named(NamedTaskStartData::TestStart(data))
    }
    pub fn test_task(data: TestTask) -> Self {
        TaskStartData::Named(NamedTaskStartData::TestTask(data))
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TaskProgressParams {
    /** Unique id of the task with optional reference to parent task id */
    pub task_id: TaskId,

    /** Timestamp of when the progress event was generated in milliseconds since Epoch. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_time: Option<i64>,

    /** Message describing the task progress.
    Information about the state of the task at the time the event is sent. */
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

    Where dataKind is: kind of data to expect in the `data` field. If this field is not set,
    the kind of data is not specified. Kind names for specific tasks like compile, test,
    etc are specified in the protocol. Data kind options specified in task_data_kind module
    and data is: Optional metadata about the task. Objects for specific tasks like compile, test,
    etc are specified in the protocol. */
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub data: Option<TaskProgressData>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "dataKind", content = "data")]
pub enum NamedTaskProgressData {}

/** Task progress notifications may contain an arbitrary interface in their `data`
field. The kind of interface that is contained in a notification must be
specified in the `dataKind` field. */
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TaskProgressData {
    Named(NamedTaskProgressData),
    Other(OtherData),
}

impl TaskProgressData {}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
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

    Where dataKind is: kind of data to expect in the `data` field. If this field is not set,
    the kind of data is not specified. Kind names for specific tasks like compile, test,
    etc are specified in the protocol. Data kind options specified in task_data_kind module
    and data is: Optional metadata about the task. Objects for specific tasks like compile, test,
    etc are specified in the protocol. */
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub data: Option<TaskFinishData>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "dataKind", content = "data")]
pub enum NamedTaskFinishData {
    CompileReport(CompileReport),
    TestFinish(TestFinish),
    TestReport(TestReport),
}

/** Task finish notifications may contain an arbitrary interface in their `data`
field. The kind of interface that is contained in a notification must be
specified in the `dataKind` field.

There are predefined kinds of objects for compile and test tasks, as described
in [[bsp#BuildTargetCompile]] and [[bsp#BuildTargetTest]] */
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TaskFinishData {
    Named(NamedTaskFinishData),
    Other(OtherData),
}

impl TaskFinishData {
    pub fn compile_report(data: CompileReport) -> Self {
        TaskFinishData::Named(NamedTaskFinishData::CompileReport(data))
    }
    pub fn test_finish(data: TestFinish) -> Self {
        TaskFinishData::Named(NamedTaskFinishData::TestFinish(data))
    }
    pub fn test_report(data: TestReport) -> Self {
        TaskFinishData::Named(NamedTaskFinishData::TestReport(data))
    }
}

/** The beginning of a compilation unit may be signalled to the client with a build/taskStart
notification. When the compilation unit is a build target, the notification's dataKind field
must be "compile-task" and the data field must include a CompileTask object. */
#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct CompileTask {
    pub target: BuildTargetIdentifier,
}

/** The completion of a compilation task should be signalled with a build/taskFinish notification.
When the compilation unit is a build target, the notification's dataKind field must be
compile-report and the data field must include a CompileReport object. */
#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
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
    pub time: Option<i64>,

    /** The compilation was a noOp compilation. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_op: Option<bool>,
}

/** The beginning of a testing unit may be signalled to the client with a build/taskStart notification.
When the testing unit is a build target, the notification's dataKind field must be
test-task and the data field must include a TestTask object. */
#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct TestTask {
    pub target: BuildTargetIdentifier,
}

/** The completion of a test task should be signalled with a build/taskFinish notification.
When the testing unit is a build target, the notification's dataKind field must be
test-report and the data field must include a TestReport object. */
#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct TestReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<String>,

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
    pub time: Option<i64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TestStart {
    /** Name or description of the test. */
    pub display_name: String,

    /** Source location of the test, as LSP location. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TestFinish {
    /** Name or description of the test. */
    pub display_name: String,

    /** Information about completion of the test, for example an error message. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /** Completion status of the test. */
    pub status: TestStatus,

    /** Source location of the test, as LSP location. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,

    /** Optionally, structured metadata about the test completion.
     * For example: stack traces, expected/actual values. */
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub data: Option<TestFinishData>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "dataKind", content = "data")]
pub enum NamedTestFinishData {}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TestFinishData {
    Named(NamedTestFinishData),
    Other(OtherData),
}

impl TestFinishData {}

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub uri: URI,
    pub range: Range,
}

#[derive(Debug, PartialEq, Serialize_repr, Deserialize_repr, Default, Clone, Copy)]
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
          "status": 2,
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
          "status": 2
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
            "status": 2
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
            origin_id: Some("test_originId".to_string()),
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
                uri: "file:///test".to_string(),
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
                uri: "file:///test".to_string(),
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
          "status": 2,
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
          "status": 2
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
