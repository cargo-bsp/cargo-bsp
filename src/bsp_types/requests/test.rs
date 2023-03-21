use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Command;

use crate::bsp_types::requests::{CreateCommand, Request};
use crate::bsp_types::BuildTargetIdentifier;

#[derive(Debug)]
pub enum Test {}

impl Request for Test {
    type Params = TestParams;
    type Result = TestResult;
    const METHOD: &'static str = "buildTarget/test";
}

impl CreateCommand for TestParams {
    fn origin_id(&self) -> Option<String> {
        self.origin_id.clone()
    }

    fn create_command(&self) -> Command {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TestParams {
    /** A sequence of build targets to test. */
    pub targets: Vec<BuildTargetIdentifier>,

    /** A unique identifier generated by the client to identify this request.
     * The server may include this id in triggered notifications or responses. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<String>,

    /** Optional arguments to the test execution engine. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<String>>,

    /** Kind of data to expect in the `data` field. If this field is not set, the kind of data is not specified. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_kind: Option<String>,

    /** Language-specific metadata about for this test execution.
     * See ScalaTestParams as an example. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TestResult {
    /** An optional request id to know the origin of this report. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<String>,

    /** A status code for the execution. */
    pub status_code: i32,

    /** Kind of data to expect in the `data` field. If this field is not set, the kind of data is not specified. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_kind: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}
