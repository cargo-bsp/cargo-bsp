use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestReport {
    #[deprecated(note = "Use the field in TaskFinishParams instead")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<Identifier>,
    /// The build target that was compiled.
    pub target: BuildTargetIdentifier,
    /// The total number of successful tests.
    pub passed: i32,
    /// The total number of failed tests.
    pub failed: i32,
    /// The total number of ignored tests.
    pub ignored: i32,
    /// The total number of cancelled tests.
    pub cancelled: i32,
    /// The total number of skipped tests.
    pub skipped: i32,
    /// The total number of milliseconds tests take to run (e.g. doesn't include compile times).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn test_report() {
        #[allow(deprecated)]
        let test_data = TestReport {
            origin_id: None,
            target: BuildTargetIdentifier::default(),
            passed: 1,
            failed: 2,
            ignored: 3,
            cancelled: 4,
            skipped: 5,
            time: Some(6),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "target": {
            "uri": ""
          },
          "passed": 1,
          "failed": 2,
          "ignored": 3,
          "cancelled": 4,
          "skipped": 5,
          "time": 6
        }
        "#
        );
        assert_json_snapshot!(TestReport::default(),
            @r#"
        {
          "target": {
            "uri": ""
          },
          "passed": 0,
          "failed": 0,
          "ignored": 0,
          "cancelled": 0,
          "skipped": 0
        }
        "#
        );
    }
}
