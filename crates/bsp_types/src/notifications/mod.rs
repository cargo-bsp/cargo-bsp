use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::Identifier;
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
mod print;
mod publish_diagnostics;
mod read_stdin;
mod tasks;

pub trait Notification {
    type Params: DeserializeOwned + Serialize;
    const METHOD: &'static str;
}

/// The Task Id allows clients to _uniquely_ identify a BSP task and establish a client-parent relationship with another task id.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskId {
    /// A unique identifier
    pub id: Identifier,
    /// The parent task ids, if any. A non-empty parents field means
    /// this task is a sub-task of every parent task id. The child-parent
    /// relationship of tasks makes it possible to render tasks in
    /// a tree-like user interface or inspect what caused a certain task
    /// execution.
    /// OriginId should not be included in the parents field, there is a separate
    /// field for that.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parents: Option<Vec<Identifier>>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    /// Line position in a document (zero-based).
    pub line: i32,
    /// Character offset on a line in a document (zero-based)
    ///
    /// If the character value is greater than the line length it defaults back
    /// to the line length.
    pub character: i32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    /// The range's start position.
    pub start: Position,
    /// The range's end position.
    pub end: Position,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use super::*;

    #[test]
    fn task_id() {
        let test_data = TaskId {
            id: "test_id".into(),
            parents: Some(vec!["test_parent".into()]),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "id": "test_id",
          "parents": [
            "test_parent"
          ]
        }
        "#
        );
        assert_json_snapshot!(TaskId::default(),
            @r#"
        {
          "id": ""
        }
        "#
        );
    }
}
