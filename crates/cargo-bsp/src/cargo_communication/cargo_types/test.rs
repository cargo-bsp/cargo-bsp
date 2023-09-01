//! Structures for parsing and handling test information from
//! `cargo +nightly test --show-output -Z unstable-options --format=json`
//! command.

use serde::{Deserialize, Serialize};

use bsp_types::notifications::{TaskDataWithKind, TestFinishData, TestReportData, TestStatus};
use bsp_types::BuildTargetIdentifier;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum TestType {
    Suite(SuiteEvent),
    Test(TestEvent),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "event")]
pub enum SuiteEvent {
    Started(SuiteStarted),
    Ok(SuiteResults),
    Failed(SuiteResults),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SuiteStarted {
    pub(crate) test_count: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SuiteResults {
    pub(crate) passed: i32,
    pub(crate) failed: i32,
    pub(crate) ignored: i32,
    /// Measured tests (benchmarks) are not yet supported by BSP.
    pub(crate) measured: i32,
    pub(crate) filtered_out: i32,
    pub(crate) exec_time: f64,
}

impl SuiteResults {
    pub fn to_test_report(&self, target: BuildTargetIdentifier) -> TaskDataWithKind {
        TaskDataWithKind::TestReport(TestReportData {
            origin_id: None,
            target,
            passed: self.passed,
            failed: self.failed,
            ignored: self.ignored,
            cancelled: 0,
            skipped: self.filtered_out,
            time: Some((self.exec_time * 1000.0) as i64),
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "event")]
pub enum TestEvent {
    Started(TestName),
    Ok(TestResult),
    Failed(TestResult),
    Ignored(TestResult),
    Timeout(TestResult),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TestName {
    pub(crate) name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TestResult {
    pub(crate) name: String,
    pub(crate) stdout: Option<String>,
}

impl TestResult {
    /// Split test output into stdout and stderr (to delete, if cargo starts
    /// sending stderr for tests).
    pub fn handle_test_stdout(&mut self) -> Option<String> {
        let mut true_stdout = String::default();
        self.stdout = self.stdout.as_ref().and_then(|stdout| {
            if let Some((out, err)) = stdout.rsplit_once("thread '") {
                true_stdout = out.to_string();
                Some(format!("thread '{}", err))
            } else {
                true_stdout = stdout.clone();
                None
            }
        });
        (!true_stdout.is_empty()).then_some(true_stdout)
    }

    pub fn map_to_test_notification(self, status: TestStatus) -> TestFinishData {
        TestFinishData {
            display_name: self.name,
            message: self.stdout,
            status,
            // TODO add location of build target
            location: None,
            data_kind: None,
            data: None,
        }
    }
}
