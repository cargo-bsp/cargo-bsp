use serde::{Deserialize, Serialize};

use crate::*;

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "dataKind", content = "data")]
pub enum NamedTaskFinishData {
    CompileReport(CompileReport),
    TestFinish(TestFinish),
    TestReport(TestReport),
}

/// Task finish notifications may contain an arbitrary interface in their `data`
/// field. The kind of interface that is contained in a notification must be
/// specified in the `dataKind` field.
///
/// There are predefined kinds of objects for compile and test tasks, as described
/// in [[bsp#BuildTargetCompile]] and [[bsp#BuildTargetTest]]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TaskFinishData {
    Named(NamedTaskFinishData),
    Other(OtherData),
}

impl TaskFinishData {
    pub fn compile_report(data: CompileReport) -> Self {
        Self::Named(NamedTaskFinishData::CompileReport(data))
    }
    pub fn test_finish(data: TestFinish) -> Self {
        Self::Named(NamedTaskFinishData::TestFinish(data))
    }
    pub fn test_report(data: TestReport) -> Self {
        Self::Named(NamedTaskFinishData::TestReport(data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn task_finish_data() {
        assert_json_snapshot!(TaskFinishData::compile_report(CompileReport::default()),
            @r#"
        {
          "dataKind": "compile-report",
          "data": {
            "target": {
              "uri": ""
            },
            "errors": 0,
            "warnings": 0
          }
        }
        "#
        );
        assert_json_snapshot!(TaskFinishData::test_report(TestReport::default()),
            @r#"
        {
          "dataKind": "test-report",
          "data": {
            "target": {
              "uri": ""
            },
            "passed": 0,
            "failed": 0,
            "ignored": 0,
            "cancelled": 0,
            "skipped": 0
          }
        }
        "#
        );
        assert_json_snapshot!(TaskFinishData::test_finish(TestFinish::default()),
            @r#"
        {
          "dataKind": "test-finish",
          "data": {
            "displayName": "",
            "status": 1
          }
        }
        "#
        );
    }
}
