use serde::{Deserialize, Serialize};

mod initialize_notifications;
pub use initialize_notifications::*;

mod messages_notifications;
pub use messages_notifications::*;

mod task_notifications;
pub use task_notifications::*;

mod build_target_notification;
pub use build_target_notification::*;

mod publish_diagnostics_notification;
pub use publish_diagnostics_notification::*;

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



