use serde::{Deserialize, Serialize};

use crate::*;

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "dataKind", content = "data")]
pub enum NamedTaskStartData {
    CompileTask(CompileTask),
    TestStart(TestStart),
    TestTask(TestTask),
}

/// Task start notifications may contain an arbitrary interface in their `data`
/// field. The kind of interface that is contained in a notification must be
/// specified in the `dataKind` field.
///
/// There are predefined kinds of objects for compile and test tasks, as described
/// in [[bsp#BuildTargetCompile]] and [[bsp#BuildTargetTest]]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TaskStartData {
    Named(NamedTaskStartData),
    Other(OtherData),
}

impl TaskStartData {
    pub fn compile_task(data: CompileTask) -> Self {
        Self::Named(NamedTaskStartData::CompileTask(data))
    }
    pub fn test_start(data: TestStart) -> Self {
        Self::Named(NamedTaskStartData::TestStart(data))
    }
    pub fn test_task(data: TestTask) -> Self {
        Self::Named(NamedTaskStartData::TestTask(data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn task_start_data() {
        assert_json_snapshot!(TaskStartData::compile_task(CompileTask::default()),
            @r#"
        {
          "dataKind": "compile-task",
          "data": {
            "target": {
              "uri": ""
            }
          }
        }
        "#
        );
        assert_json_snapshot!(TaskStartData::test_task(TestTask::default()),
            @r#"
        {
          "dataKind": "test-task",
          "data": {
            "target": {
              "uri": ""
            }
          }
        }
        "#
        );
        assert_json_snapshot!(TaskStartData::test_start(TestStart::default()),
            @r#"
        {
          "dataKind": "test-start",
          "data": {
            "displayName": ""
          }
        }
        "#
        );
    }
}
