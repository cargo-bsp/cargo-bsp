use serde::Deserialize;

use crate::bsp_types::notifications::{
    TaskDataWithKind, TestFinishData, TestReportData, TestStatus,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum TestType {
    Suite(SuiteEvent),
    Test(TestEvent),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "event")]
pub enum SuiteEvent {
    Started(SuiteStarted),
    Ok(SuiteResults),
    Failed(SuiteResults),
}

#[derive(Debug, Deserialize)]
pub struct SuiteStarted {
    #[allow(dead_code)]
    test_count: i32,
}

#[derive(Debug, Deserialize)]
pub struct SuiteResults {
    passed: i32,
    failed: i32,
    ignored: i32,
    measured: i32,
    filtered_out: i32,
    exec_time: f64,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "event")]
pub enum TestEvent {
    Started(TestName),
    Ok(TestResult),
    Failed(TestResult),
    Ignored(TestResult),
    Timeout(TestResult),
}

#[derive(Debug, Deserialize)]
pub struct TestName {
    name: String,
}

impl TestName {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug, Deserialize)]
pub struct TestResult {
    name: String,
    stdout: Option<String>,
    stderr: Option<String>,
}

impl TestResult {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn map_to_test_notification(self, status: TestStatus) -> TestFinishData {
        TestFinishData {
            display_name: self.name,
            message: self
                .stdout
                .and_then(|out| self.stderr.map(|err| format!("{}\n{}", out, err))),
            status,
            location: None,
            data_kind: None,
            data: None,
        }
    }
}
