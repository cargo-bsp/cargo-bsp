use serde::{Deserialize, Serialize};

use crate::bsp_types::notifications::{
    TaskDataWithKind, TestFinishData, TestReportData, TestStatus,
};

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
    pub(crate) measured: i32,
    pub(crate) filtered_out: i32,
    pub(crate) exec_time: f64,
}

impl SuiteResults {
    pub fn to_test_report(&self) -> TaskDataWithKind {
        TaskDataWithKind::TestReport(TestReportData {
            // TODO change target to actual BuildTargetIdentifier
            target: Default::default(),
            passed: self.passed + self.measured,
            failed: self.failed,
            ignored: self.ignored,
            cancelled: 0,
            skipped: self.filtered_out,
            time: Some((self.exec_time * 1000.0) as i32),
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
    pub(crate) stderr: Option<String>,
}

impl TestResult {
    pub fn map_to_test_notification(self, status: TestStatus) -> TestFinishData {
        TestFinishData {
            display_name: self.name,
            /// Because of a bug in cargo, all messages from tests go to stdout.
            /// When fixed, send stdout as logMessage and stderr as message below.
            message: self
                .stdout
                .and_then(|out| self.stderr.map(|err| format!("{}\n{}", out, err))),
            status,
            // TODO add location of build target
            location: None,
            data_kind: None,
            data: None,
        }
    }
}
