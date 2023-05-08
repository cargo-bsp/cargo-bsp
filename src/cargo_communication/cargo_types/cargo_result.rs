use crate::bsp_types::requests::{CompileResult, RunResult, TestResult};
use crate::bsp_types::StatusCode;

pub trait CargoResult {
    fn create_result(origin_id: Option<String>, status_code: StatusCode) -> Self;
}

impl CargoResult for CompileResult {
    fn create_result(origin_id: Option<String>, status_code: StatusCode) -> Self {
        CompileResult {
            origin_id,
            status_code,
            data_kind: None,
            data: None,
        }
    }
}

impl CargoResult for RunResult {
    fn create_result(origin_id: Option<String>, status_code: StatusCode) -> Self {
        RunResult {
            origin_id,
            status_code,
        }
    }
}

impl CargoResult for TestResult {
    fn create_result(origin_id: Option<String>, status_code: StatusCode) -> Self {
        TestResult {
            origin_id,
            status_code,
            data_kind: None,
            data: None,
        }
    }
}
