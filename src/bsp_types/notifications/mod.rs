use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

pub use did_change_build_target::*;
pub use exit_build::*;
pub use initialized_build::*;
pub use messages::*;
pub use publish_diagnostics::*;
pub use tasks::*;

mod did_change_build_target;
mod exit_build;
mod initialized_build;
mod messages;
mod publish_diagnostics;
mod tasks;

pub trait Notification {
    type Params: DeserializeOwned + Serialize;
    const METHOD: &'static str;
}

/* Included in notifications of tasks or requests to signal the completion state. */
#[derive(Debug, PartialEq, Serialize_repr, Deserialize_repr, Default, Clone)]
#[repr(u8)]
pub enum StatusCode {
    /** Execution was successful. */
    Ok = 1,
    /** Execution failed. */
    #[default]
    Error = 2,
    /** Execution was cancelled. */
    Cancelled = 3,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct TaskId {
    /** A unique identifier */
    pub id: String,

    /** The parent task ids, if any. A non-empty parents field means
     * this task is a sub-task of every parent task id. The child-parent
     * relationship of tasks makes it possible to render tasks in
     * a tree-like user interface or inspect what caused a certain task
     * execution. */
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parents: Vec<String>,
}

#[cfg(test)]
mod tests {
    use crate::bsp_types::tests::test_serialization;

    use super::*;

    #[test]
    fn status_code() {
        test_serialization(&StatusCode::Ok, r#"1"#);
        test_serialization(&StatusCode::Error, r#"2"#);
        test_serialization(&StatusCode::Cancelled, r#"3"#);
    }

    #[test]
    fn task_id() {
        let test_data = TaskId {
            id: "test_id".to_string(),
            parents: vec!["parent1".to_string(), "parent2".to_string()],
        };

        test_serialization(
            &test_data,
            r#"{"id":"test_id","parents":["parent1","parent2"]}"#,
        );

        let mut modified = test_data.clone();
        modified.parents = vec![];
        test_serialization(&modified, r#"{"id":"test_id"}"#);
    }
}
