//! CargoResult trait implementation for the Compile/Run/TestResult.
//! Allows creating the result for the client regardless if its the compile,
//! run or test request.

use bsp_types::requests::{CompileResult, RunResult, TestResult};

pub trait CargoResult {
    fn create_result(origin_id: Option<String>, status_code: i32) -> Self;
}

impl CargoResult for CompileResult {
    fn create_result(origin_id: Option<String>, status_code: i32) -> Self {
        CompileResult {
            origin_id,
            status_code,
            data_kind: None,
            data: None,
        }
    }
}

impl CargoResult for RunResult {
    fn create_result(origin_id: Option<String>, status_code: i32) -> Self {
        RunResult {
            origin_id,
            status_code,
        }
    }
}

impl CargoResult for TestResult {
    fn create_result(origin_id: Option<String>, status_code: i32) -> Self {
        TestResult {
            origin_id,
            status_code,
            data_kind: None,
            data: None,
        }
    }
}
