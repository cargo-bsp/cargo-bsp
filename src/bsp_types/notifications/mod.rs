use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

mod initialized_build;
pub use initialized_build::*;

mod exit_build;
pub use exit_build::*;

mod messages;
pub use messages::*;

mod tasks;
pub use tasks::*;

mod did_change_build_target;
pub use did_change_build_target::*;

mod publish_diagnostics;
pub use publish_diagnostics::*;

pub trait Notification {
    type Params: DeserializeOwned + Serialize;
    const METHOD: &'static str;
}

/* Included in notifications of tasks or requests to signal the completion state. */
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum StatusCode {
    /** Execution was successful. */
    Ok = 1,
    /** Execution failed. */
    #[default]
    Error = 2,
    /** Execution was cancelled. */
    Cancelled = 3,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskId {
    /** A unique identifier */
    pub id: String,

    /** The parent task ids, if any. A non-empty parents field means
     * this task is a sub-task of every parent task id. The child-parent
     * relationship of tasks makes it possible to render tasks in
     * a tree-like user interface or inspect what caused a certain task
     * execution. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parents: Option<Vec<String>>,
}
