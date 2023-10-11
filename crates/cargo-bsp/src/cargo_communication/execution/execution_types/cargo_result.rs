//! CargoResult trait implementation for the Compile/Run/TestResult.
//! Allows creating the result for the client regardless if its the compile,
//! run or test request.

use bsp_types::requests::{CompileResult, RunResult, TestResult};
use bsp_types::{Identifier, StatusCode};

pub trait CargoResult {
    fn create_result(origin_id: Option<Identifier>, status_code: StatusCode) -> Self;
}

impl CargoResult for CompileResult {
    fn create_result(origin_id: Option<Identifier>, status_code: StatusCode) -> Self {
        CompileResult {
            origin_id,
            status_code,
            data: None,
        }
    }
}

impl CargoResult for RunResult {
    fn create_result(origin_id: Option<Identifier>, status_code: StatusCode) -> Self {
        RunResult {
            origin_id,
            status_code,
        }
    }
}

impl CargoResult for TestResult {
    fn create_result(origin_id: Option<Identifier>, status_code: StatusCode) -> Self {
        TestResult {
            origin_id,
            status_code,
            data: None,
        }
    }
}
