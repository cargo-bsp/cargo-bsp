use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::notifications::{Notification, TaskId};
use crate::{BuildTargetIdentifier, StatusCode};

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
    pub data: Option<TaskDataWithKind>,
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
    pub data: Option<TaskDataWithKind>,
}

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
    pub data: Option<TaskDataWithKind>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case", tag = "dataKind", content = "data")]
pub enum TaskDataWithKind {
    CompileTask(CompileTaskData),
    CompileReport(CompileReportData),
    TestTask(TestTaskData),
    TestReport(TestReportData),
    TestStart(TestStartData),
    TestFinish(TestFinishData),
}

/** The beginning of a compilation unit may be signalled to the client with a build/taskStart
notification. When the compilation unit is a build target, the notification's dataKind field
must be "compile-task" and the data field must include a CompileTask object. */
#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct CompileTaskData {
    pub target: BuildTargetIdentifier,
}

/** The completion of a compilation task should be signalled with a build/taskFinish notification.
When the compilation unit is a build target, the notification's dataKind field must be
compile-report and the data field must include a CompileReport object. */
#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompileReportData {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_op: Option<bool>,
}

/** The beginning of a testing unit may be signalled to the client with a build/taskStart notification.
When the testing unit is a build target, the notification's dataKind field must be
test-task and the data field must include a TestTask object. */
#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct TestTaskData {
    pub target: BuildTargetIdentifier,
}

/** The completion of a test task should be signalled with a build/taskFinish notification.
When the testing unit is a build target, the notification's dataKind field must be
test-report and the data field must include a TestReport object. */
#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
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

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TestStartData {
    /** Name or description of the test. */
    pub display_name: String,

    /** Source location of the test, as LSP location. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TestFinishData {
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

    /** Kind of data to expect in the `data` field. If this field is not set, the kind of data is not specified. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_kind: Option<String>,

    /** Optionally, structured metadata about the test completion.
     * For example: stack traces, expected/actual values. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

pub type Location = lsp_types::Location;

#[derive(Debug, PartialEq, Serialize_repr, Deserialize_repr, Default, Clone)]
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
    use url::Url;

    use super::*;

    #[test]
    fn task_start_method() {
        assert_eq!(TaskStart::METHOD, "build/taskStart");
    }

    #[test]
    fn task_progress_method() {
        assert_eq!(TaskProgress::METHOD, "build/taskProgress");
    }

    #[test]
    fn task_finish_method() {
        assert_eq!(TaskFinish::METHOD, "build/taskFinish");
    }

    #[test]
    fn task_start_params() {
        let test_data = TaskStartParams {
            task_id: TaskId::default(),
            event_time: Some(1),
            message: Some("test_message".to_string()),
            data: Some(TaskDataWithKind::CompileTask(CompileTaskData::default())),
        };

        assert_json_snapshot!(test_data,
            @r###"
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
        "###
        );
        assert_json_snapshot!(TaskStartParams::default(),
            @r###"
        {
          "taskId": {
            "id": ""
          }
        }
        "###
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
            data: Some(TaskDataWithKind::CompileTask(CompileTaskData::default())),
        };

        assert_json_snapshot!(test_data,
            @r###"
        {
          "taskId": {
            "id": ""
          },
          "eventTime": 1,
          "message": "test_message",
          "total": 2,
          "progress": 3,
          "unit": "test_unit",
          "dataKind": "compile-task",
          "data": {
            "target": {
              "uri": ""
            }
          }
        }
        "###
        );
        assert_json_snapshot!(TaskProgressParams::default(),
            @r###"
        {
          "taskId": {
            "id": ""
          }
        }
        "###
        );
    }

    #[test]
    fn task_finish_params() {
        let test_data = TaskFinishParams {
            task_id: TaskId::default(),
            event_time: Some(1),
            message: Some("test_message".to_string()),
            status: StatusCode::default(),
            data: Some(TaskDataWithKind::CompileTask(CompileTaskData::default())),
        };

        assert_json_snapshot!(test_data,
            @r###"
        {
          "taskId": {
            "id": ""
          },
          "eventTime": 1,
          "message": "test_message",
          "status": 2,
          "dataKind": "compile-task",
          "data": {
            "target": {
              "uri": ""
            }
          }
        }
        "###
        );
        assert_json_snapshot!(TaskFinishParams::default(),
            @r###"
        {
          "taskId": {
            "id": ""
          },
          "status": 2
        }
        "###
        );
    }

    #[test]
    fn task_data_with_kind() {
        assert_json_snapshot!(TaskDataWithKind::CompileTask(CompileTaskData::default()),
            @r###"
        {
          "dataKind": "compile-task",
          "data": {
            "target": {
              "uri": ""
            }
          }
        }
        "###
        );
        assert_json_snapshot!(TaskDataWithKind::CompileReport(CompileReportData::default()),
            @r###"
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
        "###
        );
        assert_json_snapshot!(TaskDataWithKind::TestTask(TestTaskData::default()),
            @r###"
        {
          "dataKind": "test-task",
          "data": {
            "target": {
              "uri": ""
            }
          }
        }
        "###
        );
        assert_json_snapshot!(TaskDataWithKind::TestReport(TestReportData::default()),
            @r###"
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
        "###
        );
        assert_json_snapshot!(TaskDataWithKind::TestStart(TestStartData::default()),
            @r###"
        {
          "dataKind": "test-start",
          "data": {
            "displayName": ""
          }
        }
        "###
        );
        assert_json_snapshot!(TaskDataWithKind::TestFinish(TestFinishData::default()),
            @r###"
        {
          "dataKind": "test-finish",
          "data": {
            "displayName": "",
            "status": 2
          }
        }
        "###
        );
    }

    #[test]
    fn compile_task_data() {
        assert_json_snapshot!(CompileTaskData::default(),
            @r###"
        {
          "target": {
            "uri": ""
          }
        }
        "###
        );
    }

    #[test]
    fn compile_report_data() {
        let test_data = CompileReportData {
            target: BuildTargetIdentifier::default(),
            origin_id: Some("test_originId".to_string()),
            errors: 1,
            warnings: 2,
            time: Some(3),
            no_op: Some(true),
        };

        assert_json_snapshot!(test_data,
            @r###"
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
        "###
        );
        assert_json_snapshot!(CompileReportData::default(),
            @r###"
        {
          "target": {
            "uri": ""
          },
          "errors": 0,
          "warnings": 0
        }
        "###
        );
    }

    #[test]
    fn test_task_data() {
        assert_json_snapshot!(TestTaskData::default(),
            @r###"
        {
          "target": {
            "uri": ""
          }
        }
        "###
        );
    }

    #[test]
    fn test_report_data() {
        let test_data = TestReportData {
            target: BuildTargetIdentifier::default(),
            passed: 1,
            failed: 2,
            ignored: 3,
            cancelled: 4,
            skipped: 5,
            time: Some(6),
        };

        assert_json_snapshot!(test_data,
            @r###"
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
        "###
        );
        assert_json_snapshot!(TestReportData::default(),
            @r###"
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
        "###
        );
    }

    #[test]
    fn test_start_data() {
        let test_data = TestStartData {
            display_name: "test_name".to_string(),
            location: Some(Location::new(
                Url::from_file_path("/test").unwrap(),
                lsp_types::Range::default(),
            )),
        };

        assert_json_snapshot!(test_data,
            @r###"
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
        "###
        );
        assert_json_snapshot!(TestStartData::default(),
            @r###"
        {
          "displayName": ""
        }
        "###
        );
    }

    #[test]
    fn test_finish_data() {
        let test_data = TestFinishData {
            display_name: "test_name".to_string(),
            message: Some("test_message".to_string()),
            status: TestStatus::default(),
            location: Some(Location::new(
                Url::from_file_path("/test").unwrap(),
                lsp_types::Range::default(),
            )),
            data_kind: Some("test_dataKind".to_string()),
            data: Some(serde_json::json!({"dataKey": "dataValue"})),
        };

        assert_json_snapshot!(test_data,
            @r###"
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
        "###
        );
        assert_json_snapshot!(TestFinishData::default(),
            @r###"
        {
          "displayName": "",
          "status": 2
        }
        "###
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
